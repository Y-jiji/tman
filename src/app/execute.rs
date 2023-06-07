pub enum ArgPattern {
    Word(regex::Regex),
    Variable(regex::Regex),
}

pub struct CommandExecution<App> {
    pub pattern: Vec<ArgPattern>,
    pub execute: fn(&mut App, Vec<&str>, &mut crate::DataBase) -> Result<(), String>,
}

impl<App> CommandExecution<App> {
    pub fn try_execute(&self, command: &Vec<&str>, app: &mut App, db: &mut crate::DataBase) -> Result<bool, String> {
        let will_execute = command.len() == self.pattern.len() && (0..command.len()).map(|i| 
            match self.pattern[i] {
                ArgPattern::Word(ref r) => r.is_match(command[i]),
                ArgPattern::Variable(ref r) => r.is_match(command[i])
            }
        ).fold(true, |x, y| x && y);
        if !will_execute { return Ok(false) }
        let args = command.iter().enumerate().filter_map(|(i, x)| matches!(self.pattern[i], ArgPattern::Word(_)).then_some(*x));
        (self.execute)(app, args.collect(), db).map(|()| true)
    }
}

macro_rules! x_decl {
    ($($x:ident $y:literal, )* |$z0: ident, $z1: ident, $z2: ident| $body: tt) => {
        CommandExecution {
            pattern: vec![$(x_decl!{$x $y}, )*],
            execute: |$z0, $z1, $z2| $body,
        }
    };
    ($(($($x:tt)*))*) => {
        vec![ $(x_decl!{$($x)*}, )* ]
    };
    (w $x:literal) => {
        ArgPattern::Word(regex::Regex::new($x).unwrap())
    };
    (v $x:literal) => {
        ArgPattern::Variable(regex::Regex::new($x).unwrap())
    };
}