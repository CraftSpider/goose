
Values are `dyn ValItem` boxes, where a `ValItem` is a type that defines the behavior
of contained data. A `Type` is runtime information about a `ValItem`. An extension should have once-initialized
`Type`s for each `ValItem`, then, these will get set as globals during init. A user referring to a type
refers to the `Type` stored in the scope.
