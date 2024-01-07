Example output:

<pre>
$ <b>cargo run && cargo run --release</b>

   <i>Compiling quote v1.0.35</i>
   <i>Compiling quote-benchmark v0.0.0</i>
<kbd><kbd><b>macro in debug mode: 440 micros</b></kbd></kbd>
    <i>Finished dev [unoptimized + debuginfo] target(s) in 4.39s</i>
     <i>Running `target/debug/quote-benchmark`</i>
<kbd><kbd><b>non-macro in debug mode: 537 micros</b></kbd></kbd>
   <i>Compiling quote v1.0.35</i>
   <i>Compiling quote-benchmark v0.0.0</i>
<kbd><kbd><b>macro in release mode: 423 micros</b></kbd></kbd>
    <i>Finished release [optimized] target(s) in 4.00s</i>
     <i>Running `target/release/quote-benchmark`</i>
<kbd><kbd><b>non-macro in release mode: 134 micros</b></kbd></kbd>
</pre>
