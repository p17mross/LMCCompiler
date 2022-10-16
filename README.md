
# LMCCompiler

A project for converting a pseudocode-like language to Little Man Computer assembly, written in rust.

  

## Use

Run the project with the first command line argument being the path to a file to read the program from.

The resulting assembly will be printed to stdout.

  

## Syntax of language

### Comments:

    //This is a comment
    a = b //You can put comments after lines of code, too


  

### Variables:

    a = 10 //Sets a to 10
    b = a //Sets b to the value of a
    c = b + 10 //Sets c to b + 10

  

### Input / output


    input a //Gets a number from the user and stores it in a
    output a //Outputs the value of a
    print a //Same as above
    print a + 10 //Prints a + 10

  
### If statements

    input a
    if a > 10
         print 10 //Indentation is not required
    else if a > 0 //Can have none of these or as many as required
         print 0
    else //Also optional
         print 100
    endif //Is required


 
input a

    input b
    if a == b //Conditionals are not chainable: e.g. a == b && a == 10 is not possible
         if a == 10
              print 10
         else
              print 20
         endif
    else
         print 30
    endif

  

### While loops

  
    input a
    while a > 0
         a = a -1
         print a
    endwhile //Required

You can also use the special keywords `true` and `break`

    while true //True is only for while loops, not if statements or variables
             input a
         if a == 10
             break //Breaks inner most loop
         endif
    endwhile


### Operators

#### Arithmetic operators

*  `+` for addition

*  `-` for subtraction

Multiplication / division are not implemented as they would require more complicated logic

#### Comparison operators

*  `==` for equality

*  `!=` for inequality

*  `>`, `<`, `>=`, `<=` for comparison


### Not implemented

* Functions / procedures as they require a stack

* Arrays / lists as they require pointers (I'm not doing self modifying code)

* For loops because I'm lazy
