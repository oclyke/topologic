# topologic
This module provides a structure for representing and manipulating acyclic dependency graphs. Usage is generally to build a graph of dependencies by adding direct dependency pairs and then querying the graph for properties such as:
* total dependencies of a certain node
* total dependents of a certain node
* topological sorting of dependencies
* topological sorting of dependents

# Usage
Please see the unit tests in `lib.rs` for examples of usage.

# Todo:
* Change topographical sort to use flags instead of memory allocation to avoid cloning the graph.
