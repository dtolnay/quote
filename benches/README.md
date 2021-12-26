Example output:

<pre>
$ <b>cargo run && cargo run --release</b>

   <i>Compiling quote v1.0.10</i>
   <i>Compiling quote-benchmark v0.0.0</i>
<kbd><kbd><b>macro in debug mode: 1655 micros</b></kbd></kbd>
    <i>Finished dev [unoptimized + debuginfo] target(s) in 4.39s</i>
     <i>Running `/git/quote/target/debug/quote-benchmark`</i>
<kbd><kbd><b>non-macro in debug mode: 1205 micros</b></kbd></kbd>
   <i>Compiling quote v1.0.10</i>
   <i>Compiling quote-benchmark v0.0.0</i>
<kbd><kbd><b>macro in release mode: 1635 micros</b></kbd></kbd>
    <i>Finished release [optimized] target(s) in 4.00s</i>
     <i>Running `/git/quote/target/release/quote-benchmark`</i>
<kbd><kbd><b>non-macro in release mode: 105 micros</b></kbd></kbd>
</pre>
