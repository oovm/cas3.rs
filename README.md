# cas3.rs

https://reference.wolfram.com/language/tutorial/EvaluationOfExpressions.html

https://reference.wolfram.com/language/tutorial/TheInternalsOfTheWolframSystem.html
https://reference.wolfram.com/language/tutorial/SomeNotesOnInternalImplementation.html

"The Wolfram Language is an infinite evaluation system. When you enter an expression, the Wolfram Language will keep on using definitions it knows until it gets a result to which no definitions apply."

"In the standard evaluation procedure, the Wolfram System first evaluates the head of an expression and then evaluates each element of the expression. These elements are in general themselves expressions, to which the same evaluation procedure is recursively applied."

https://mathematica.stackexchange.com/questions/96/what-is-the-distinction-between-downvalues-upvalues-subvalues-and-ownvalues

https://mathematica.stackexchange.com/questions/192278/is-there-a-defined-priority-for-pattern-matching
https://reference.wolfram.com/language/tutorial/TransformationRulesAndDefinitions.html#26982

https://reference.wolfram.com/language/tutorial/Evaluation.html


this links to a book that describes the evaluation order (section 7.1.3)
https://mathematica.stackexchange.com/questions/16485/are-you-interested-in-purchasing-david-wagners-power-programming-with-mathemat


combinators 
https://writings.stephenwolfram.com/2020/12/combinators-a-centennial-view/
https://writings.stephenwolfram.com/2020/12/combinators-and-the-story-of-computation/

```mathematica
k[x_][y_] := x
s[x_][y_][z_] := x[z][y[z]]

or 
EXPR //. {s[x_][y_][z_] -> x[z][y[z]], k[x_][y_] -> x}
```

random todo - not critical for combinator reduction

* improve parser (whitespace and EOF robustness) - infix/m-expr if im feeling naughty
* arb numerics- switch to rug/gmp for all number types 
* string escaping 
* levels have a simple structure that spans wl. like replace with {{}} makes a list applying rules separately, 
* having x_Head syntax would be really nice 



```scheme
(set xb (pattern x (blank)))
(set yb (pattern y (blank)))
(set zb (pattern z (blank)))

(set l1 (((s xb) yb) zb))
(set r1 ((x z) (y z)))
(set rule1 (rule l1 r1))
(set l2 ((k xb) yb))
(set r2 x)
(set rule2 (rule l2 r2))
(set crules (list rule1 rule2))



(set ex ((((s s) k) k) k))

```

s[s][k][k][k]

((((s s) k) k) k)

s and k rules

(list (rule (((s (pattern x (blank))) (pattern y (blank))) (pattern z (blank))) ((x z) (y z))) (rule ((k (pattern x (blank))) (pattern y (blank))) x))

(rr ((((s s) k) k) k) (list (rule (((s (pattern x (blank))) (pattern y (blank))) (pattern z (blank))) ((x z) (y z))) (rule ((k (pattern x (blank))) (pattern y (blank))) x)))

the goal is to make this reduce to 

(((s (pattern x (blank))) (pattern y (blank))) (pattern z (blank)))

(((s (pattern x (blank))) (pattern y (blank))) (pattern z (blank)))

(In 28) := (replace_all ((((s s) k) k) k) (rule (((s (pattern x (blank))) (pattern y (blank))) (pattern z (blank))) ((x z) (y z))))
(Out 28) = (((x z) (y z)) k)