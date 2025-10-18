use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "api-test-app")]
#[command(about = "A comprehensive test app for the Switchboard API")]
#[command(version = "1.0")]
struct Cli {
    #[arg(long, default_value = "http://localhost:3030")]
    api_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test user creation and authentication
    TestUsers {
        #[arg(long)]
        count: u32,
    },
    /// Test chat creation and management
    TestChats {
        #[arg(long)]
        user_token: String,
        #[arg(long)]
        count: u32,
    },
    /// Test complete workflow
    TestWorkflow {
        #[arg(long)]
        user_count: u32,
        #[arg(long)]
        chats_per_user: u32,
        #[arg(long)]
        messages_per_chat: u32,
    },
    /// Run all tests
    RunAll,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: i64,
    public_id: String,
    email: Option<String>,
    display_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionResponse {
    token: String,
    user: User,
}

#[derive(Debug, Serialize, Deserialize)]
struct Chat {
    id: i64,
    public_id: String,
    user_id: i64,
    folder_id: Option<i64>,
    title: String,
    chat_type: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    id: i64,
    public_id: String,
    chat_id: i64,
    user_id: i64,
    content: String,
    message_type: String,
    role: String,
    model: Option<String>,
    thread_id: Option<i64>,
    reply_to_id: Option<i64>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateChatRequest {
    title: String,
    chat_type: Option<String>,
    folder_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateMessageRequest {
    content: String,
    role: String,
    message_type: Option<String>,
    model: Option<String>,
    thread_id: Option<String>,
    reply_to_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Provider {
    id: i64,
    name: String,
    api_base: String,
    models: Vec<String>,
    created_at: String,
    updated_at: String,
}

struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    fn new(base_url: String) -> Self {
        let client = Client::new();
        Self { client, base_url }
    }

    async fn create_user(&self, email: &str, display_name: &str) -> Result<SessionResponse> {
        println!("=5 Creating user: {} ({})", display_name, email);

        let mut user_data = HashMap::new();
        user_data.insert("email", email);
        user_data.insert("display_name", display_name);

        let response = self
            .client
            .post(format!("{}/api/auth/dev-token", self.base_url))
            .json(&user_data)
            .send()
            .await
            .context("Failed to create user")?;

        if response.status() != StatusCode::OK {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Failed to create user: {}", error));
        }

        let session_response: SessionResponse = response.json().await
            .context("Failed to parse session response")?;

        println!(" User created successfully: {}", session_response.user.public_id.green());
        Ok(session_response)
    }

    async fn list_chats(&self, token: &str) -> Result<Vec<Chat>> {
        println!("=Ë Fetching chats for user");

        let response = self
            .client
            .get(format!("{}/api/chats", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to fetch chats")?;

        if response.status() != StatusCode::OK {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Failed to fetch chats: {}", error));
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct ChatsResponse {
            chats: Vec<Chat>,
        }

        let chats_response: ChatsResponse = response.json().await
            .context("Failed to parse chats response")?;

        println!("=Ê Found {} chats", chats_response.chats.len().to_string().yellow());
        Ok(chats_response.chats)
    }

    async fn create_chat(&self, token: &str, request: CreateChatRequest) -> Result<Chat> {
        println!("<• Creating chat: {}", request.title);

        let response = self
            .client
            .post(format!("{}/api/chats", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&request)
            .send()
            .await
            .context("Failed to create chat")?;

        if response.status() != StatusCode::OK {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Failed to create chat: {}", error));
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct ChatResponse {
            chat: Chat,
        }

        let chat_response: ChatResponse = response.json().await
            .context("Failed to parse chat response")?;

        println!(" Chat created: {}", chat_response.chat.public_id.green());
        Ok(chat_response.chat)
    }

    async fn get_chat(&self, token: &str, chat_id: &str) -> Result<Chat> {
        println!("= Fetching chat: {}", chat_id);

        let response = self
            .client
            .get(format!("{}/api/chats/{}", self.base_url, chat_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to fetch chat")?;

        if response.status() != StatusCode::OK {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Failed to fetch chat: {}", error));
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct ChatDetailResponse {
            chat: Chat,
        }

        let chat_response: ChatDetailResponse = response.json().await
            .context("Failed to parse chat response")?;

        Ok(chat_response.chat)
    }

    async fn send_message(&self, token: &str, chat_id: &str, message: CreateMessageRequest) -> Result<Message> {
        println!("=¬ Sending message to chat: {}", chat_id);

        let response = self
            .client
            .post(format!("{}/api/chats/{}/messages", self.base_url, chat_id))
            .header("Authorization", format!("Bearer {}", token))
            .json(&message)
            .send()
            .await
            .context("Failed to send message")?;

        if response.status() != StatusCode::OK {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Failed to send message: {}", error));
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct MessageResponse {
            message: Message,
        }

        let message_response: MessageResponse = response.json().await
            .context("Failed to parse message response")?;

        println!(" Message sent: {}", message_response.message.public_id.green());
        Ok(message_response.message)
    }

    async fn get_messages(&self, token: &str, chat_id: &str) -> Result<Vec<Message>> {
        println!("=Ü Fetching messages for chat: {}", chat_id);

        let response = self
            .client
            .get(format!("{}/api/chats/{}/messages", self.base_url, chat_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to fetch messages")?;

        if response.status() != StatusCode::OK {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Failed to fetch messages: {}", error));
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct MessagesResponse {
            messages: Vec<Message>,
        }

        let messages_response: MessagesResponse = response.json().await
            .context("Failed to parse messages response")?;

        println!("=Ê Found {} messages", messages_response.messages.len().to_string().yellow());
        Ok(messages_response.messages)
    }

    async fn configure_provider(&self, token: &str, provider_name: &str, api_base: &str, models: Vec<String>) -> Result<Provider> {
        println!("™ Configuring provider: {}", provider_name);

        let mut provider_data = HashMap::new();
        provider_data.insert("name", provider_name);
        provider_data.insert("api_base", api_base);

        // Convert Vec<String> to a JSON string for serialization
        let models_json = serde_json::to_string(&models)?;
        provider_data.insert("models", &models_json);

        let response = self
            .client
            .post(format!("{}/api/providers", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&provider_data)
            .send()
            .await
            .context("Failed to configure provider")?;

        // Note: This endpoint may not exist yet, but we're testing it
        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                let provider: Provider = response.json().await
                    .context("Failed to parse provider response")?;
                println!(" Provider configured: {}", provider.name.green());
                Ok(provider)
            }
            status => {
                let error = response.text().await?;
                println!("  Provider configuration returned {}: {}", status, error);
                Err(anyhow::anyhow!("Provider configuration failed: {}", error))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let api_client = ApiClient::new(cli.api_url);

    println!("=€ Switchboard API Test App");
    println!("========================");

    match cli.command {
        Commands::TestUsers { count } => {
            test_user_creation(&api_client, count).await?;
        }
        Commands::TestChats { user_token, count } => {
            test_chat_management(&api_client, &user_token, count).await?;
        }
        Commands::TestWorkflow { user_count, chats_per_user, messages_per_chat } => {
            test_complete_workflow(&api_client, user_count, chats_per_user, messages_per_chat).await?;
        }
        Commands::RunAll => {
            run_all_tests(&api_client).await?;
        }
    }

    println!("\n All tests completed successfully!");
    Ok(())
}

async fn test_user_creation(api_client: &ApiClient, count: u32) -> Result<()> {
    println!("\n>ê Testing User Creation");
    println!("=======================");

    let mut users = Vec::new();
    for i in 1..=count {
        let email = format!("testuser{}@example.com", i);
        let display_name = format!("Test User {}", i);

        match api_client.create_user(&email, &display_name).await {
            Ok(session) => {
                users.push(session);
            }
            Err(e) => {
                println!("L Failed to create user {}: {}", i, e);
            }
        }
    }

    println!(" Successfully created {}/{} users", users.len(), count);
    Ok(())
}

async fn test_chat_management(api_client: &ApiClient, user_token: &str, count: u32) -> Result<()> {
    println!("\n>ê Testing Chat Management");
    println!("==========================");

    let existing_chats = api_client.list_chats(user_token).await?;
    println!("=Ê User currently has {} chats", existing_chats.len());

    let mut created_chats = Vec::new();
    for i in 1..=count {
        let chat_request = CreateChatRequest {
            title: format!("Test Chat {}", i),
            chat_type: Some("direct".to_string()),
            folder_id: None,
        };

        match api_client.create_chat(user_token, chat_request).await {
            Ok(chat) => {
                created_chats.push(chat);
            }
            Err(e) => {
                println!("L Failed to create chat {}: {}", i, e);
            }
        }
    }

    // Test fetching individual chats
    for chat in &created_chats {
        match api_client.get_chat(user_token, &chat.public_id).await {
            Ok(_) => {
                println!(" Successfully fetched chat: {}", chat.title.green());
            }
            Err(e) => {
                println!("L Failed to fetch chat {}: {}", chat.public_id, e);
            }
        }
    }

    println!(" Successfully created {} chats", created_chats.len());
    Ok(())
}

async fn test_complete_workflow(api_client: &ApiClient, user_count: u32, chats_per_user: u32, messages_per_chat: u32) -> Result<()> {
    println!("\n>ê Testing Complete Workflow");
    println!("===========================");

    // Step 1: Create users
    println!("\n=Ý Step 1: Creating {} users", user_count);
    let mut users = Vec::new();
    for i in 1..=user_count {
        let email = format!("workflowuser{}@example.com", i);
        let display_name = format!("Workflow User {}", i);

        match api_client.create_user(&email, &display_name).await {
            Ok(session) => {
                users.push(session);
            }
            Err(e) => {
                println!("L Failed to create user {}: {}", i, e);
            }
        }
    }

    if users.is_empty() {
        return Err(anyhow::anyhow!("No users created, cannot continue workflow"));
    }

    // Step 2: Create chats for each user
    println!("\n=¬ Step 2: Creating {} chats per user", chats_per_user);
    let mut all_chats = Vec::new();

    for (user_idx, user) in users.iter().enumerate() {
        println!("Creating chats for user: {}", user.user.display_name.as_ref().unwrap_or(&format!("User {}", user_idx + 1)));

        for chat_idx in 1..=chats_per_user {
            let chat_request = CreateChatRequest {
                title: format!("User {} Chat {}", user_idx + 1, chat_idx),
                chat_type: Some("direct".to_string()),
                folder_id: None,
            };

            match api_client.create_chat(&user.token, chat_request).await {
                Ok(chat) => {
                    all_chats.push((user.token.clone(), chat));
                }
                Err(e) => {
                    println!("L Failed to create chat {} for user {}: {}", chat_idx, user_idx + 1, e);
                }
            }
        }
    }

    // Step 3: Send messages to chats
    println!("\n=è Step 3: Sending {} messages per chat", messages_per_chat);
    let mut message_count = 0;

    for (token, chat) in &all_chats {
        for msg_idx in 1..=messages_per_chat {
            let message_request = CreateMessageRequest {
                content: format!("This is test message {} in chat '{}'", msg_idx, chat.title),
                role: "user".to_string(),
                message_type: Some("text".to_string()),
                model: None,
                thread_id: None,
                reply_to_id: None,
            };

            match api_client.send_message(token, &chat.public_id, message_request).await {
                Ok(_) => {
                    message_count += 1;
                }
                Err(e) => {
                    println!("L Failed to send message {} to chat {}: {}", msg_idx, chat.public_id, e);
                }
            }
        }
    }

    // Step 4: Verify messages
    println!("\n= Step 4: Verifying messages");
    let mut verified_messages = 0;

    for (token, chat) in &all_chats {
        match api_client.get_messages(token, &chat.public_id).await {
            Ok(messages) => {
                verified_messages += messages.len();
                println!("=Ê Chat '{}' has {} messages", chat.title, messages.len());
            }
            Err(e) => {
                println!("L Failed to verify messages for chat {}: {}", chat.public_id, e);
            }
        }
    }

    // Step 5: Test provider configuration (for first user)
    println!("\n™ Step 5: Testing provider configuration");
    if let Some(first_user) = users.first() {
        let models = vec![
            "gpt-3.5-turbo".to_string(),
            "gpt-4".to_string(),
            "claude-3-sonnet".to_string(),
        ];

        match api_client.configure_provider(
            &first_user.token,
            "OpenAI",
            "https://api.openai.com/v1",
            models,
        ).await {
            Ok(_) => {
                println!(" Provider configuration test completed");
            }
            Err(e) => {
                println!("  Provider configuration test failed (this may be expected): {}", e);
            }
        }
    }

    // Summary
    println!("\n=Ê Workflow Summary");
    println!("===================");
    println!(" Users created: {}/{}", users.len(), user_count);
    println!(" Chats created: {}", all_chats.len());
    println!(" Messages sent: {}", message_count);
    println!(" Messages verified: {}", verified_messages);

    Ok(())
}

async fn run_all_tests(api_client: &ApiClient) -> Result<()> {
    println!(">ê Running All Tests");
    println!("==================");

    // Test 1: User creation
    test_user_creation(api_client, 3).await?;

    // Create a user for subsequent tests
    let user_session = api_client.create_user("mainuser@example.com", "Main Test User").await?;

    // Test 2: Chat management
    test_chat_management(api_client, &user_session.token, 5).await?;

    // Test 3: Complete workflow
    test_complete_workflow(api_client, 2, 3, 2).await?;

    println!("\n<‰ All tests completed successfully!");
    Ok(())
}