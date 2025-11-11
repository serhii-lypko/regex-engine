 In a deterministic automaton, every state has exactly one transition for each possible input. In a non-deterministic automaton, an input can lead to one, more than one, or no transition for a given state. The powerset construction algorithm can transform any nondeterministic automaton into a (usually more complex) deterministic automaton with identical functionality.
------------------------------------------------------------------------------------------


The properties of DFA:
- each of its transitions is uniquely determined by its source state and input symbol, and
- reading an input symbol is required for each state transition.


DFA vs NFA

Machine A:
State S1, on input 'a' → goes to S2
State S1, on input 'b' → goes to S3

Machine B:
State S1, on input 'a' → goes to S2
State S1, on input 'a' → ALSO goes to S3

In Machine A: if you're in S1 and see 'a', where do you go? One answer.
In Machine B: if you're in S1 and see 'a', where do you go? Two possible answers.

