
# Installation

Install goose by moving to the project root directory and running `./install.sh`. This will place a
`goose` executable at the project root, which can be copied to the desired location.

# Your first program

To get started with goose, create a file named `example.hnk`. Then, copy and paste the following code into it:

```goose
write(console, "Honk!")
```

Running `goose example.hnk` should then print out "Honk!" to the command line and exit. Congratulations,
you've run your first program.

# Comments

Goose has two forms of comments. Single line and multi-line. Both use `$` as the base character, with single-line
comments going from any single `$` to the end of the line, and multi-line comments going from any set of three `$` in
a row till an ending set of three `$`

```goose
$ I'm a single-line comment!

$$$
    I'm a multi-line comment,
    see?
$$$
```

# Basic Types

With your first program running, it's time to introduce the basic types available in goose. You've already
used one - a `chararray`, defined as a sequence of characters between two quotes. Some other basic types you may
encounter are `int`, `float`, `bit`, and `char`. Below are some examples simple literals for each of these types. 

```goose
$ An integer, or int, is a whole number, positive or negative
1;
-789;
$ A float is any number which contains a decimal
1.0;
-2.5;
10.;
$ A bit is either 1 or 0, you'll see this type later used in limits and comparisons
0b;
1b;
$ A character, or char, is a single character. More strictly, it's a unicode code-point, which may or may not
$ be enough to display anything useful on its own
'a';
'-';
$ A chararray is a bunch of characters all in a row, representing a line or lines of text.
"Hello, goose";
$ An array is a set of values of the same type
[1, 2, 3];
["a", "b", "c"];
```

The above aren't the only value types you might see, though they are the simplest. We'll get to the other types later.

# Defining a variable

A variable in goose stores a value for later use. When you first define a variable, you must choose whether it will
be 'unique' or 'carryover'. For now, there's no functional difference between the two, however, it will be important
to remember the distinction once we get to functions.

```goose
unique honk = "Honk!";
carryover goose = "Goose says...";

write(console, goose);
write(console, honk);
```

It's also important to note that variables in goose can't change type after they've first been defined. If you
try to assign a variable with the wrong type, it will throw an exception.

```goose
unique honk = "Honk!";
honk = 5; $ Exception! Honk expects type chararray.
```

# Defining a function

Time for the most important thing in goose - functions. Functions in goose are strictly typed, in that they
must define their return and call types, and if passed the wrong type, will throw an exception. Every function has a
'limit' condition, which is tested after every statement until it becomes true. Once it does, the value of the last
executed statement is returned as the result of the function. If the limit doesn't become true by the end of the
function, the function starts over from the beginning, redefining any `unique` variables, keeping any `carryover`
variables from the last execution.

```goose
def fn name: int (a: int, b: int) -> |b > a| {
    a -= 1;
}
```

# Control flow

The primary thing that makes goose unique is that it has no explicit conditionals or loop statements. Instead,
function limits and the fact that they implicitly loop should be used to control flow through the function.
If you wanted, you might define an `if` function that looks like this:

```goose
fn if: null (cond: bool, block: fn: null ()) -> |!cond| [
    block();
    unique cond = 0b;
]
```
You would then invoke your new function by calling it with an expression that evaluates to a bit (A value that is 
either 0 or 1), and a closure (an unnamed, inline function) to execute if the condition is true. Note that closures
also require limits, so we define this one with one which becomes true at the end of the closure.

```goose
if(1 == 1, fn: null () -> |stop| [
    write(console, "Honk!")
    unique stop = 1b
])
```
