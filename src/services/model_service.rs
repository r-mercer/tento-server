use openai_func_enums::ToolSet;

#[derive(Debug, ToolSet)]
pub enum FunctionDef {
    CreateQuiz {},

    GPT { prompt: String },
}
