extern crate proc_macro;

use proc_macro::token_stream::IntoIter as TokenIter;
use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use std::collections::VecDeque;
use std::iter::FromIterator;

struct Context {
    out_token: TokenTree,
    span_token: TokenTree,
    default_span: bool,
}

#[proc_macro_derive(QuoteImpl)]
pub fn quote_impl(input: TokenStream) -> TokenStream {
    expand(input, true)
}

#[proc_macro_derive(QuoteSpannedImpl)]
pub fn quote_spanned_impl(input: TokenStream) -> TokenStream {
    expand(input, false)
}

fn expand(input: TokenStream, default_span: bool) -> TokenStream {
    // Input looks like:
    //
    //    enum QuoteHack {
    //        Value = (stringify! { $out $span $($tt)* }, 0).1,
    //    }
    //
    //println!("{:#?}", input);

    //use std::time::Instant;
    //let begin = Instant::now();
    let mut input = input.into_iter();

    input.next().unwrap(); // enum
    input.next().unwrap(); // QuoteHack
    let mut input = match input.next().unwrap() {
        TokenTree::Group(group) => group.stream().into_iter(),
        _ => unreachable!(),
    };

    input.next().unwrap(); // Value
    input.next().unwrap(); // =
    let mut input = match input.next().unwrap() {
        TokenTree::Group(group) => group.stream().into_iter(),
        _ => unreachable!(),
    };

    input.next().unwrap(); // stringify
    input.next().unwrap(); // !
    let mut input = match input.next().unwrap() {
        TokenTree::Group(group) => group.stream().into_iter(),
        _ => unreachable!(),
    };

    let ctx = Context {
        out_token: input.next().unwrap(),
        span_token: input.next().unwrap(),
        default_span,
    };

    let mut content = Vec::new();
    let mut interp = Vec::new();
    quote_each_token(input, &ctx, &mut content, &mut interp);
    let ret = wrap_in_macro_rule(content);

    //let dur = begin.elapsed();
    //let dur = dur.as_secs() * 1_000_000 + dur.subsec_micros() as u64;
    //println!("proc-quote {} micros", dur);
    ret
}

fn wrap_in_macro_rule(content: Vec<TokenTree>) -> TokenStream {
    /*
        macro_rules! proc_macro_call {
            () => {{
                $content
            }}
        }
    */

    TokenStream::from_iter(vec![
        TokenTree::Ident(Ident::new("macro_rules", Span::call_site())),
        TokenTree::Punct(Punct::new('!', Spacing::Alone)),
        TokenTree::Ident(Ident::new("proc_macro_call", Span::call_site())),
        TokenTree::Group(Group::new(
            Delimiter::Brace,
            TokenStream::from_iter(vec![
                TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
                TokenTree::Punct(Punct::new('=', Spacing::Joint)),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Group(Group::new(
                    Delimiter::Brace,
                    TokenStream::from_iter(vec![TokenTree::Group(Group::new(
                        Delimiter::Brace,
                        TokenStream::from_iter(content),
                    ))]),
                )),
            ]),
        )),
    ])
}

fn quote_fully(input: TokenIter, ctx: &Context, out: &mut Vec<TokenTree>, interp: &mut Vec<Ident>) {
    /*
        {
            let mut __qs = $crate::__rt::TokenStream::new();
            $input...
            __qs
        }
    */

    let ctx = Context {
        out_token: TokenTree::Ident(Ident::new("__qs", Span::call_site())),
        span_token: ctx.span_token.clone(),
        default_span: ctx.default_span,
    };

    let mut content = vec![
        TokenTree::Ident(Ident::new("let", Span::call_site())),
        TokenTree::Ident(Ident::new("mut", Span::call_site())),
        TokenTree::Ident(Ident::new("__qs", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Ident(Ident::new("quote", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("__rt", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("TokenStream", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("new", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ];
    quote_each_token(input, &ctx, &mut content, interp);
    content.push(TokenTree::Ident(Ident::new("__qs", Span::call_site())));

    out.push(TokenTree::Group(Group::new(
        Delimiter::Brace,
        TokenStream::from_iter(content),
    )));
}

fn quote_each_token(mut input: TokenIter, ctx: &Context, out: &mut Vec<TokenTree>, interp: &mut Vec<Ident>) {
    let mut reparse = VecDeque::new();
    loop {
        let token = if let Some(token) = reparse.pop_front() {
            token
        } else if let Some(token) = input.next() {
            token
        } else {
            return;
        };
        match token {
            TokenTree::Punct(punct) => {
                if punct.as_char() == '#' {
                    match input.next() {
                        Some(TokenTree::Ident(var)) => {
                            interp.push(var.clone());
                            quote_interpolation(out, &ctx, var);
                        }
                        Some(TokenTree::Group(group)) => {
                            if group.delimiter() == Delimiter::Parenthesis {
                                match parse_separator(&mut input) {
                                    Separator::Valid(sep) => {
                                        let stream = group.stream();
                                        quote_repetition(out, &ctx, stream, sep, interp);
                                    }
                                    Separator::Invalid(undo) => {
                                        quote_punct(out, &ctx, punct);
                                        reparse.push_back(TokenTree::Group(group));
                                        reparse.extend(undo);
                                    }
                                }
                            } else {
                                quote_punct(out, &ctx, punct);
                                reparse.push_back(TokenTree::Group(group));
                            }
                        }
                        next => {
                            quote_punct(out, &ctx, punct);
                            reparse.extend(next);
                        }
                    }
                } else {
                    quote_punct(out, &ctx, punct);
                }
            }
            TokenTree::Ident(ident) => quote_ident(out, &ctx, ident),
            TokenTree::Group(group) => quote_group(out, &ctx, group, interp),
            TokenTree::Literal(literal) => quote_literal(out, &ctx, literal),
        }
    }
}

enum Separator {
    Valid(Vec<TokenTree>),
    Invalid(Vec<TokenTree>),
}

fn parse_separator(input: &mut TokenIter) -> Separator {
    let mut sep = Vec::new();
    let mut seen_alone = false;
    while let Some(token) = input.next() {
        match token {
            TokenTree::Punct(punct) => {
                let alone = punct.spacing() == Spacing::Alone;
                if punct.as_char() == '*' && alone {
                    return Separator::Valid(sep);
                }
                sep.push(TokenTree::Punct(punct));
                if seen_alone && alone {
                    return Separator::Invalid(sep);
                }
                seen_alone |= alone;
            }
            non_punct => {
                sep.push(non_punct);
                if seen_alone {
                    return Separator::Invalid(sep);
                }
                seen_alone = true;
            }
        }
    }
    Separator::Invalid(sep)
}

fn quote_ident(out: &mut Vec<TokenTree>, ctx: &Context, ident: Ident) {
    let string = ident.to_string();
    if string.starts_with("r#") {
        quote_ident_raw(out, ctx, string);
    } else {
        quote_ident_plain(out, ctx, string);
    }
}

fn quote_ident_plain(out: &mut Vec<TokenTree>, ctx: &Context, ident: String) {
    /*
        $crate::TokenStreamExt::append(&mut $out, $crate::__rt::Ident::new($ident, $span));
    */

    out.extend(vec![
        TokenTree::Ident(Ident::new("quote", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("TokenStreamExt", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("append", Span::call_site())),
        TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            TokenStream::from_iter(vec![
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Ident(Ident::new("mut", Span::call_site())),
                ctx.out_token.clone(),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Ident(Ident::new("quote", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("__rt", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Ident", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("new", Span::call_site())),
                TokenTree::Group(Group::new(
                    Delimiter::Parenthesis,
                    TokenStream::from_iter(vec![
                        TokenTree::Literal(Literal::string(&ident)),
                        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                        ctx.span_token.clone(),
                    ]),
                )),
            ]),
        )),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);
}

fn quote_ident_raw(out: &mut Vec<TokenTree>, ctx: &Context, ident: String) {
    /*
        $crate::__rt::parse(&mut $out, $span, $ident);
    */

    out.extend(vec![
        TokenTree::Ident(Ident::new("quote", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("__rt", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("parse", Span::call_site())),
        TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            TokenStream::from_iter(vec![
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Ident(Ident::new("mut", Span::call_site())),
                ctx.out_token.clone(),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                ctx.span_token.clone(),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Literal(Literal::string(&ident)),
            ]),
        )),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);
}

fn quote_punct(out: &mut Vec<TokenTree>, ctx: &Context, punct: Punct) {
    /*
        let mut __q = $crate::__rt::Punct::new($ch, $crate::__rt::Spacing::$spacing);
        __q.set_span($span);
        $crate::TokenStreamExt::append(&mut $out, __q);
    */

    let ch = punct.as_char();
    let spacing = match punct.spacing() {
        Spacing::Joint => "Joint",
        Spacing::Alone => "Alone",
    };

    out.extend(vec![
        TokenTree::Ident(Ident::new("let", Span::call_site())),
        TokenTree::Ident(Ident::new("mut", Span::call_site())),
        TokenTree::Ident(Ident::new("__q", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Ident(Ident::new("quote", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("__rt", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("Punct", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("new", Span::call_site())),
        TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            TokenStream::from_iter(vec![
                TokenTree::Literal(Literal::character(ch)),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Ident(Ident::new("quote", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("__rt", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Spacing", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new(spacing, Span::call_site())),
            ]),
        )),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);

    if !ctx.default_span {
        out.extend(vec![
            TokenTree::Ident(Ident::new("__q", Span::call_site())),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("set_span", Span::call_site())),
            TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                TokenStream::from_iter(vec![ctx.span_token.clone()]),
            )),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);
    }

    out.extend(vec![
        TokenTree::Ident(Ident::new("quote", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("TokenStreamExt", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("append", Span::call_site())),
        TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            TokenStream::from_iter(vec![
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Ident(Ident::new("mut", Span::call_site())),
                ctx.out_token.clone(),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Ident(Ident::new("__q", Span::call_site())),
            ]),
        )),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);
}

fn quote_group(out: &mut Vec<TokenTree>, ctx: &Context, group: Group, interp: &mut Vec<Ident>) {
    /*
        let mut __q = $crate::__rt::Group::new(
            $crate::__rt::Delimiter::Parenthesis,
            $content...
        );
        __q.set_span($span);
        $crate::TokenStreamExt::append(&mut $out, __q);
    */

    let delimiter = match group.delimiter() {
        Delimiter::Parenthesis => "Parenthesis",
        Delimiter::Bracket => "Bracket",
        Delimiter::Brace => "Brace",
        Delimiter::None => "None",
    };

    out.extend(vec![
        TokenTree::Ident(Ident::new("let", Span::call_site())),
        TokenTree::Ident(Ident::new("mut", Span::call_site())),
        TokenTree::Ident(Ident::new("__q", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Ident(Ident::new("quote", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("__rt", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("Group", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("new", Span::call_site())),
        TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            TokenStream::from_iter({
                let mut inner = vec![
                    TokenTree::Ident(Ident::new("quote", Span::call_site())),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("__rt", Span::call_site())),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("Delimiter", Span::call_site())),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new(delimiter, Span::call_site())),
                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                ];
                quote_fully(group.stream().into_iter(), &ctx, &mut inner, interp);
                inner
            }),
        )),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);

    if !ctx.default_span {
        out.extend(vec![
            TokenTree::Ident(Ident::new("__q", Span::call_site())),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("set_span", Span::call_site())),
            TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                TokenStream::from_iter(vec![ctx.span_token.clone()]),
            )),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);
    }

    out.extend(vec![
        TokenTree::Ident(Ident::new("quote", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("TokenStreamExt", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("append", Span::call_site())),
        TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            TokenStream::from_iter(vec![
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Ident(Ident::new("mut", Span::call_site())),
                ctx.out_token.clone(),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Ident(Ident::new("__q", Span::call_site())),
            ]),
        )),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);
}

fn quote_literal(out: &mut Vec<TokenTree>, ctx: &Context, literal: Literal) {
    /*
        $crate::__rt::parse(&mut $out, $span, $literal);
    */

    out.extend(vec![
        TokenTree::Ident(Ident::new("quote", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("__rt", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("parse", Span::call_site())),
        TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            TokenStream::from_iter(vec![
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Ident(Ident::new("mut", Span::call_site())),
                ctx.out_token.clone(),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                ctx.span_token.clone(),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Literal(Literal::string(&literal.to_string())),
            ]),
        )),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);
}

fn quote_interpolation(out: &mut Vec<TokenTree>, ctx: &Context, var: Ident) {
    /*
        $crate::ToTokens::to_tokens(&$var, &mut $out);
    */

    out.extend(vec![
        TokenTree::Ident(Ident::new("quote", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("ToTokens", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("to_tokens", Span::call_site())),
        TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            TokenStream::from_iter(vec![
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Ident(var),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Ident(Ident::new("mut", Span::call_site())),
                ctx.out_token.clone(),
            ]),
        )),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);
}

fn quote_repetition(
    out: &mut Vec<TokenTree>,
    ctx: &Context,
    content: TokenStream,
    sep: Vec<TokenTree>,
    interp_outer: &mut Vec<Ident>,
) {
    /*
        for ((a, b), c) in a.into_iter().zip(b).zip(c) {
            $content...
        }

        for (_i, ((a, b), c)) in a.into_iter().zip(b).zip(c).enumerate() {
            if _i > 0 {
                $sep...
            }
            $content...
        }
    */

    let process = content.into_iter();
    let mut content = Vec::new();
    let mut interp = Vec::new();
    quote_each_token(process, ctx, &mut content, &mut interp);
    let enumerate = !sep.is_empty() && !interp.is_empty();

    out.extend(vec![
        TokenTree::Ident(Ident::new("for", Span::call_site())),
        nested_tuples_pat(&interp, enumerate),
        TokenTree::Ident(Ident::new("in", Span::call_site())),
    ]);
    multi_zip_expr(&interp, out, enumerate);
    interp_outer.extend(interp);

    let body = if enumerate {
        let sep_iter = TokenStream::from_iter(sep).into_iter();
        let mut sep = Vec::new();
        let mut interp_sep = Vec::new();
        quote_each_token(sep_iter, ctx, &mut sep, &mut interp_sep);
        let mut body = vec![
            TokenTree::Ident(Ident::new("if", Span::call_site())),
            TokenTree::Ident(Ident::new("_i", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Literal(Literal::usize_suffixed(0)),
            TokenTree::Group(Group::new(Delimiter::Brace, TokenStream::from_iter(sep))),
        ];
        body.extend(content);
        body
    } else {
        content
    };
    out.push(TokenTree::Group(Group::new(Delimiter::Brace, TokenStream::from_iter(body))));
}

fn nested_tuples_pat(interp: &[Ident], enumerate: bool) -> TokenTree {
    if interp.is_empty() {
        return TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new()));
    }
    let mut pat = TokenTree::Ident(interp[0].clone());
    for ident in &interp[1..] {
        pat = TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::from_iter(vec![
            pat,
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(ident.clone()),
        ])));
    }
    if enumerate {
        pat = TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::from_iter(vec![
            TokenTree::Ident(Ident::new("_i", Span::call_site())),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            pat,
        ])));
    }
    pat
}

fn multi_zip_expr(interp: &[Ident], out: &mut Vec<TokenTree>, enumerate: bool) {
    if interp.is_empty() {
        out.extend(vec![
            TokenTree::Punct(Punct::new('&', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Bracket, TokenStream::new())),
        ]);
        return;
    }
    let single = interp.len() == 1;
    let mut vars = interp.iter();
    out.push(TokenTree::Ident(vars.next().unwrap().clone()));
    if single && !enumerate {
        return;
    }
    out.extend(vec![
        TokenTree::Punct(Punct::new('.', Spacing::Alone)),
        TokenTree::Ident(Ident::new("into_iter", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
    ]);
    for var in vars {
        out.extend(vec![
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("zip", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::from_iter(vec![
                TokenTree::Ident(var.clone()),
            ]))),
        ]);
    }
    if enumerate {
        out.extend(vec![
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("enumerate", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
        ]);
    }
}
