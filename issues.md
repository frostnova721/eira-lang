# Issues
## List the issues and details here!

- ### Bytecode Indexing issue when release is used in a particular way [UNPATCHED]
    ```
    spell isPrime(a: NumWeave) {
        fate a <= 1 {
            chant "false";
            release;
        }   
    }
    cast isPrime with 10;
    ```
    Consider this sample. The program just gives:

    `thread 'main' panicked at src/runtime/vm.rs:29:44:
    index out of bounds: the len is 25 but the index is 25`
    error. 

    This issue is observed when a single release is used in a different scope (such as fate in this case) and the release is left un declared in the scope of the spell. 