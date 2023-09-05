pub struct Compiler {
    state: CompilerState,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            state: CompilerState::Any,
        }
    }
}

enum CompilerState {
    // General states
    //
    
    /// Any func definition
    Any,

    /// Any func definition except the main function
    AnyExceptMain,

    // Func definition states
    //
    
    /// We hit a `function` definition, now we want the identifier.
    ExpectingFuncIdentifier,

    /// We hit the function identifier, now we want the left paranthesis for the arguments.
    ExpectingFuncLeftParanthesis,

    /// We hit the function paranthesis for arguments, now we want the actual 
    /// arguments which can be malleable or not.
    ExpectingFuncMalleableOrIdentifier,

    /// We hit a malleable arg definition, now we want the identified.
    ExpectingFuncMalleableIdentifier,

    /// We have hit the function identifier, now we either expect a colon,
    /// in order to specify the argument type, a comma, or the right paranthesis
    /// to end the function definitions.
    ExpectingFuncColonCommaOrRightParanthesis,

    /// We finished the arguments definition, now we want the start brace
    /// of the function body.
    ExpectingFuncBrace,

    /// We hit the start brace of the function body, now we want the actual body.
    ExpectingFuncBody,

    // Func bodies
    //
}