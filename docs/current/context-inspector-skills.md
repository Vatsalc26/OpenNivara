# Context Inspector Skills

Skills participate in context through routing metadata and summaries. The
inspector-facing fields include pack id, category, route policy, allowed and
denied tool labels, exam, exam stage, freshness sensitivity, official source
labels, and best-for hints.

This makes skill routing auditable without enabling hidden behavior. A user can
inspect why an India exam prompt routed to a skill and then adjust enablement in
Settings > Skills.

Manual chat selection and session pins are also visible through routing state:
they select only enabled skills and are scoped to the current message or session.
