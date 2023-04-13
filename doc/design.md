# Design Document

The goal of this project is to create a parallel implementation of a compiler of
a general purpose programming language. This document describes the design
decisions behind the language, compiler and implementation.

# Why Bother? Don't most compilers operate in parallel already?

The minimum input a typical compiler can process is a file. As such, it can only
parallelize compilation of individual files. Even then, latter stages of a
compiler, like optimization, will often fall back on single threaded operations.
Creating a compiler that is completely parallel from lexing to code generation
is an interesting challenge.

With that said, there is no commonly known way of creating a truly parallel
compiler. It has only been shown that it is possible for languages of low
complexity with no practical use. This project aims to create a compiler that is
*practically* parallel. This means that the compiler's speed should increase
linearly with the number of threads at its disposable *up to some reasonable
limit* given a sufficiently large input. I other words, it should scale to
thousands of threads but not necessarily millions. This trade-off makes creating
such a compiler seems with the realm of possibility (at the time of writing).

## Lexer Design

The input program is first split up into chunks of arbitrary size. Since each 
chunk will be processed by its own thread, the chunks must be large enough so
that the processing overhead is less than the time taken to process each chunk.
Processing overhead generally exists from thread management and memory allocation.

The starting of each chunk must be at the end of a known lexeme according to the
lexical grammar. This is because certain lexeme's can take up more than one
character (like numbers or strings). Scanning for this boundary is done
heuristically depending on the language. In a typical program, very little
scanning is required.

> This is the first compromise that allows for the creation of pathological and
> impractical programs that can potentially slow down the compiler. Take, for
> example, a "hello world" program that prints out a string variable and exits.
> If the string is incredibly large *and* contains no place to split the string
> according to the lexical grammar, then the compiler will be forced to scan the
> whole string until it finds the lexeme denoting the end of the string. In this
> made up example, the compiler would have to scan what is effectively the
> entire program in order to find a chunk boundary. This could be fixed by
> reading the very large string from the file or embedding a file in the final
> binary during compilation. The same can be said for extremely large
> identifiers.

The next step is to pass each chunk through the lexer. Since we know that the
chunk is not inside a lexeme, we can 

## Lexer implementation

The file contents are mmap'd to memory, which is more than fast enough,
especially with caching.



