use poise::serenity_prelude::UserId;

#[derive(Default, Clone, Debug)]
pub struct CreateTag {
    pub name: String,
    pub content: String,
    pub owner: UserId,
}

impl CreateTag {
    pub fn new(
        name: impl Into<String>,
        content: impl Into<String>,
        owner: impl Into<UserId>,
    ) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
            owner: owner.into(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn owner(mut self, owner: impl Into<UserId>) -> Self {
        self.owner = owner.into();
        self
    }
}
