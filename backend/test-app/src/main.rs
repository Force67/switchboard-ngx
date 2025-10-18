use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
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
        println!("Creating user: {} ({})", display_name, email);

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

        println!("User created successfully: {}", session_response.user.public_id);
        Ok(session_response)
    }

    async fn create_chat(&self, token: &str, request: CreateChatRequest) -> Result<Chat> {
        println!("Creating chat: {}", request.title);

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

        println!("Chat created: {}", chat_response.chat.public_id);
        Ok(chat_response.chat)
    }

    async fn send_message(&self, token: &str, chat_id: &str, message: CreateMessageRequest) -> Result<Message> {
        println!("Sending message to chat: {}", chat_id);

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

        println!("Message sent: {}", message_response.message.public_id);
        Ok(message_response.message)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let api_client = ApiClient::new(cli.api_url);

    println!("Switchboard API Test App");
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

    println!("\nAll tests completed successfully!");
    Ok(())
}

async fn test_user_creation(api_client: &ApiClient, count: u32) -> Result<()> {
    println!("\nTesting User Creation");
    println!("=======================");

    for i in 1..=count {
        let email = format!("testuser{}@example.com", i);
        let display_name = format!("Test User {}", i);

        match api_client.create_user(&email, &display_name).await {
            Ok(_) => {
                println!("Created user {}: {}", i, email);
            }
            Err(e) => {
                println!("Failed to create user {}: {}", i, e);
            }
        }
    }

    Ok(())
}

async fn test_chat_management(api_client: &ApiClient, user_token: &str, count: u32) -> Result<()> {
    println!("\nTesting Chat Management");
    println!("==========================");

    for i in 1..=count {
        let chat_request = CreateChatRequest {
            title: format!("Test Chat {}", i),
            chat_type: Some("direct".to_string()),
            folder_id: None,
        };

        match api_client.create_chat(user_token, chat_request).await {
            Ok(_) => {
                println!("Created chat {}", i);
            }
            Err(e) => {
                println!("Failed to create chat {}: {}", i, e);
            }
        }
    }

    Ok(())
}

async fn test_complete_workflow(api_client: &ApiClient, user_count: u32, chats_per_user: u32, messages_per_chat: u32) -> Result<()> {
    println!("\nTesting Complete Workflow");
    println!("===========================");

    // Create a user
    let user_session = api_client.create_user("workflowuser@example.com", "Workflow User").await?;

    // Create chats
    for chat_idx in 1..=chats_per_user {
        let chat_request = CreateChatRequest {
            title: format!("Workflow Chat {}", chat_idx),
            chat_type: Some("direct".to_string()),
            folder_id: None,
        };

        match api_client.create_chat(&user_session.token, chat_request).await {
            Ok(chat) => {
                println!("Created chat: {}", chat.title);

                // Send messages
                for msg_idx in 1..=messages_per_chat {
                    let message_request = CreateMessageRequest {
                        content: format!("Test message {} in chat '{}'", msg_idx, chat.title),
                        role: "user".to_string(),
                        message_type: Some("text".to_string()),
                        model: None,
                        thread_id: None,
                        reply_to_id: None,
                    };

                    match api_client.send_message(&user_session.token, &chat.public_id, message_request).await {
                        Ok(_) => {
                            println!("Sent message {} to chat {}", msg_idx, chat_idx);
                        }
                        Err(e) => {
                            println!("Failed to send message: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to create chat {}: {}", chat_idx, e);
            }
        }
    }

    println!("Workflow test completed!");
    Ok(())
}

async fn run_all_tests(api_client: &ApiClient) -> Result<()> {
    println!("Running All Tests");
    println!("==================");

    test_user_creation(api_client, 2).await?;

    let user_session = api_client.create_user("mainuser@example.com", "Main Test User").await?;

    test_chat_management(api_client, &user_session.token, 3).await?;

    test_complete_workflow(api_client, 1, 2, 2).await?;

    println!("\nAll tests completed successfully!");
    Ok(())
}