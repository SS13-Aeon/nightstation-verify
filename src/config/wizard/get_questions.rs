use dialoguer::{console::style, theme::Theme, Input, Select};

use crate::config::Question;

use super::Wizard;

enum MenuOp {
    Continue,
    Done,
}

struct QuestionsWizard<'a, T: Theme> {
    theme: &'a T,
    questions: Vec<Question>,
}

impl<'a, T: Theme + 'a> QuestionsWizard<'a, T> {
    fn new(wizard: &'a Wizard<T>) -> QuestionsWizard<'a, T> {
        Self {
            theme: &wizard.theme,
            questions: Vec::new(),
        }
    }

    fn add_answer(&self, question: &mut Question) -> MenuOp {
        let text: String = Input::with_theme(self.theme)
            .with_prompt("Enter answer text")
            .interact_text()
            .unwrap();

        question.answers.push(text);

        MenuOp::Continue
    }

    fn remove_answer(&self, question: &mut Question) -> MenuOp {
        let options: Vec<_> = question
            .answers
            .iter()
            .enumerate()
            .map(|(i, a)| {
                if question.correct_answer == i {
                    format!("[Correct] {}", a)
                } else {
                    a.clone()
                }
            })
            .chain(std::iter::once("# Cancel".into()))
            .collect();

        let selection = Select::with_theme(self.theme)
            .with_prompt("Select answer to remove")
            .items(&options)
            .default(0)
            .interact()
            .unwrap();

        if selection < question.answers.len() {
            if selection < question.correct_answer {
                question.correct_answer -= 1;
            }
            question.answers.remove(selection);
        }

        MenuOp::Continue
    }

    fn check_answers(&self, question: &mut Question) -> MenuOp {
        if question.correct_answer >= question.answers.len() {
            eprintln!("{}", style("Question has no correct answer").bold().red());

            MenuOp::Continue
        } else {
            MenuOp::Done
        }
    }

    fn edit_question_prompt(&self, question: &mut Question) -> MenuOp {
        question.prompt = Input::with_theme(self.theme)
            .with_prompt("Enter question text")
            .default(question.prompt.clone())
            .interact_text()
            .unwrap();

        MenuOp::Continue
    }

    fn edit_question_id(&self, question: &mut Question) -> MenuOp {
        question.id = Input::with_theme(self.theme)
            .with_prompt("Enter question ID")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.as_bytes().len() > 100 {
                    Err("String must be less than 100 bytes")
                } else if self.questions.iter().any(|q| q.id == *input) {
                    Err("Question IDs must be unique")
                } else {
                    Ok(())
                }
            })
            .default(question.id.clone())
            .interact_text()
            .unwrap();

        MenuOp::Continue
    }

    fn edit_question(&self, question: &mut Question) -> MenuOp {
        let operations = [
            (
                "Add answer",
                Self::add_answer as fn(&Self, &mut Question) -> MenuOp,
            ),
            ("Remove answer", Self::remove_answer),
            ("Edit question text", Self::edit_question_prompt),
            ("Edit question ID", Self::edit_question_id),
            ("Continue", Self::check_answers),
        ];

        loop {
            let options: Vec<_> = question
                .answers
                .iter()
                .enumerate()
                .map(|(i, a)| {
                    if question.correct_answer == i {
                        format!("[Correct] {}", a)
                    } else {
                        a.clone()
                    }
                })
                .chain(operations.iter().map(|&(n, _)| format!("# {}", n)))
                .collect();

            let selection = Select::with_theme(self.theme)
                .with_prompt(format!(
                    "Select correct answer for {} ({})",
                    question.prompt, question.id
                ))
                .items(&options)
                .default(0)
                .interact()
                .unwrap();

            let op = if selection < question.answers.len() {
                question.correct_answer = selection;

                MenuOp::Continue
            } else {
                operations[selection - question.answers.len()].1(self, question)
            };

            match op {
                MenuOp::Continue => continue,
                MenuOp::Done => break,
            }
        }

        MenuOp::Continue
    }

    fn prompt_question(&self) -> (String, String) {
        let prompt: String = Input::with_theme(self.theme)
            .with_prompt("Enter question text")
            .interact_text()
            .unwrap();

        let id: String = Input::with_theme(self.theme)
            .with_prompt("Enter question ID")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.as_bytes().len() > 100 {
                    Err("String must be less than 100 bytes")
                } else if self.questions.iter().any(|q| q.id == *input) {
                    Err("Question IDs must be unique")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .unwrap();

        (prompt, id)
    }

    fn add_question(&mut self) -> MenuOp {
        let (prompt, id) = self.prompt_question();
        let mut question = Question {
            id,
            prompt,
            answers: Vec::new(),
            correct_answer: 0,
        };
        Self::edit_question(self, &mut question);
        self.questions.push(question);

        MenuOp::Continue
    }

    fn add_confirm_question(&mut self) -> MenuOp {
        let (prompt, id) = self.prompt_question();

        self.questions.push(Question {
            id,
            prompt,
            answers: vec!["Yes".into(), "No".into()],
            correct_answer: 0,
        });

        MenuOp::Continue
    }

    fn remove_question(&mut self) -> MenuOp {
        let options: Vec<_> = self
            .questions
            .iter()
            .enumerate()
            .map(|(i, q)| format!("{}. {} ({})", i + 1, q.prompt, q.id))
            .chain(std::iter::once("# Cancel".into()))
            .collect();

        let selection = Select::with_theme(self.theme)
            .with_prompt("Select question to remove")
            .items(&options)
            .default(0)
            .interact()
            .unwrap();

        if selection < self.questions.len() {
            self.questions.remove(selection);
        }

        MenuOp::Continue
    }

    fn run(mut self) -> Vec<Question> {
        let operations = [
            (
                "Add question",
                Self::add_question as fn(&mut QuestionsWizard<'a, T>) -> MenuOp,
            ),
            ("Add yes/no question", Self::add_confirm_question),
            ("Remove question", Self::remove_question),
            ("Done", |_| MenuOp::Done),
        ];

        loop {
            let options: Vec<_> = self
                .questions
                .iter()
                .enumerate()
                .map(|(i, q)| format!("{}. {} ({})", i + 1, q.prompt, q.id))
                .chain(operations.iter().map(|&(n, _)| format!("# {}", n)))
                .collect();

            let selection = Select::with_theme(self.theme)
                .with_prompt("Select question to edit")
                .items(&options)
                .default(0)
                .interact()
                .unwrap();

            let result = if selection < self.questions.len() {
                // We have to do this clone to avoid two mutable refs to self
                let mut question = self.questions[selection].clone();
                let res = Self::edit_question(&self, &mut question);
                self.questions[selection] = question;

                res
            } else {
                operations[selection - self.questions.len()].1(&mut self)
            };

            match result {
                MenuOp::Continue => continue,
                MenuOp::Done => break,
            }
        }

        self.questions
    }
}

impl<T: Theme> Wizard<T> {
    pub fn get_questions(&self) -> (String, Vec<Question>) {
        let ckey_prompt: String = Input::with_theme(&self.theme)
            .with_prompt("Enter BYOND username prompt")
            .default("What is your BYOND username?".into())
            .interact_text()
            .unwrap();

        let questions = QuestionsWizard::new(self).run();

        (ckey_prompt, questions)
    }
}
