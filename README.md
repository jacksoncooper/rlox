`rlox` is a tree-walk interpreter for Bob's Nystrom's Lox programming language
from the book _Crafting Interpreters_. It's my first Rust project, and I was
following the Java implementation, so my implementation is not particularly
clean or idiomatic. Thank you Bob Nystrom for the comfort food.

### TODO

Performance is pretty abysmal compared to the reference implementation `jlox`,
about _three times_ slower using Bob's benchmarks. I don't know what the JVM is
up to when it allocates memory, goodness. `perf` shows the overhead is in:

- `_int_malloc()` Heap allocation is expensive. 5.39 percent.
- `look_up_variable()`: Walking the environment chains is expensive, even with
   the resolver. 5.25 percent.
- `hashbrown::map::make_hash` Hashing identifiers is expensive. 5.07 percent.
- `<std::collections::hash::map::DefaultHasher as core::hash::Hasher>::write`
   Hashing is still expensive? This supplies the state for the hashing function,
   apparently. 6.60 percent.

```lox
class Bagel {
    init(flavor) {
        this.flavor = flavor;
        this.bites = 0;
        this.toasted = false;
        this.with_lox = false;
    }

    toast() {
        this.toasted = true;
    }

    add_lox() {
        this.with_lox = true;
    }

    munch() {
        var adverbs = repeat("", "very ", this.bites);
        print "This " + this.show() + " is " + adverbs + "tasty.";
        this.bites = this.bites +  1;
    }

    show() {
        var readable = "";
        if (this.toasted) readable = "toasted ";
        readable = readable + this.flavor + " bagel";
        if (this.with_lox) readable = readable + " with lox";
        return readable;
    }
}

fun repeat(accumulator, object, times) {
    for (var time = 1; time <= times; time = time + 1)
        accumulator = accumulator + object;
    return accumulator;
}

var my_breakfast = Bagel("pumpernickel");

my_breakfast.munch();
my_breakfast.toast();
my_breakfast.munch();
my_breakfast.add_lox();
my_breakfast.munch();
my_breakfast.munch();
```
