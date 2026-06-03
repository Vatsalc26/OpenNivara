# Model Registry And Token Budget

Every model API call is fresh. The model only knows what OpenNivara sends in that request, so ContextCompiler must select relevant context and fit it into the model window.

`ModelRegistry` provides:

- provider and model name
- context window tokens
- default reserved output tokens
- token counting support
- usage metadata support
- tokenizer strategy

Budget rules:

1. Reserve output tokens first.
2. Compute remaining input budget.
3. Estimate or count section tokens.
4. Include required sections.
5. Trim lower-priority sections first.
6. Record included and trimmed sections in `TokenBudgetReport`.

Priority order starts with system policy and current user message, then correction/runtime/location/tasks/memory/graph/workspace/session summaries according to relevance.
