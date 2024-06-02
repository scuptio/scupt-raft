# run fuzz

1. run fuzz testing on target
    ```
    cargo fuzz run [target]
    ```

2. Minify target corpus of input files
    ```
    cargo fuzz cmin [target]
    ```

3. Generate test case(only run case in corpus)
    ```
    cargo fuzz coverage [target]
    ```
   
   In '/tmp/[target_name]' directory, output the json case file.
