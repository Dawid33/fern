Common:
- Meta-programming through compile time executing using an interpreter.
- Well-defined implicit type conversion between corelang and other languages for
  correctly sized types.

xlang
- Assembly instructions as keywords.
- Builtin types for registers. 
- Source code structure resembles assemebly programming with goto's and labels
  for unstructured programming. Optional abilitiy to use core lang subset for
  structured programming. Percise control over binary code layout.

corelang
- Is a subset of all imperative languages and provides a common types system for
  interoperability between all languages.
- Exhaustive, namespaced enums.
- Slices.
- Meta-programming through compile time execution.

lisplang, forthlang, funclang
- Interpreted language with respective syntax and semantics. Internal type
  system ala python or tcl.
- Can call code from other languages, JIT compile the required files and insert
  them into the runtime. Optionally emit a binary with bundled interpreter and
  compiled code.


```

  
```
