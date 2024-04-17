# RUST CANISTER OPTIMIZATION 
This is a demo on how you can know the different resources that are being consumed by your canister's functions 

In this demo, we use the ``instruction_counter`` API from the ``ic_cdk`` to help us know the number of instructions (or resources) the code has consumed since the last endpoint 

We are checking the number of resources (in web assembly) consumed when we call the ``register_user`` function 

```
Line 164: 

let start = ic_cdk::api::instruction_counter();  // This line starts couting the number of instructions 
```

```
Line 197: 

let end = ic_cdk::api::instruction_counter(); // Stops counting the number of instructions 
```

```
Line 198: 

let instructions_consumed = end - start; // Finds the total instructions consumed 
```

```
Line 201:     

// Parses the instructions consumed in the "INSTRUCTIONS_CONSUMED" global variable for purposes of display 
    INSTRUCTIONS_CONSUMED.with(|instructions| {
        let mut instructions = instructions.lock().unwrap(); 
        *instructions = instructions_consumed
    }); 

````