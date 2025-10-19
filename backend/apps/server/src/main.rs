use anyhow::Context;
use clap::{Parser, Subcommand};
use sqlx::Row;
use switchboard_gateway::{build_router, GatewayState};
use switchboard_gateway::state::JwtConfig;
use switchboard_backend_runtime::{telemetry, BackendServices};
use switchboard_config::load as load_config;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tracing::info;

#[derive(Parser)]
#[command(name = "switchboard-backend")]
#[command(about = "Switchboard backend (console by default)")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP server
    Serve,
    /// Dump folders and chats from the database
    DumpData,
    /// Clear all folders and chats from the database
    ClearData,
    /// Seed the database with test data
    SeedData,
    /// Start interactive console (default)
    Console,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Console) {
        Commands::Serve => run_server().await,
        Commands::DumpData => dump_data().await,
        Commands::ClearData => clear_data().await,
        Commands::SeedData => seed_data().await,
        Commands::Console => run_console().await,
    }
}

async fn run_server() -> anyhow::Result<()> {
    telemetry::init_tracing().context("failed to initialise tracing")?;

    info!("starting Switchboard backend");

    let config = load_config().context("failed to load configuration")?;

    let services = BackendServices::initialise(&config)
        .await
        .context("failed to initialise backend services")?;

    let jwt_config = JwtConfig::default();
    let state = GatewayState::new(
        services.db_pool.clone(),
        jwt_config,
    );
    let app = build_router(state);

    let address = format!("{}:{}", config.http.address, config.http.port);
    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("failed to bind http listener on {address}"))?;

    info!(%address, "http server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(switchboard_backend_runtime::shutdown_signal())
        .await
        .context("http server error")?;

    info!("backend shut down");
    Ok(())
}

async fn dump_data() -> anyhow::Result<()> {
    telemetry::init_tracing().context("failed to initialise tracing")?;

    info!("dumping chat folders from database");

    let config = load_config().context("failed to load configuration")?;

    let services = BackendServices::initialise(&config)
        .await
        .context("failed to initialise backend services")?;

    use sqlx::Row;

    // Dump folders
    let folders = sqlx::query(
        r#"
        SELECT id, public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at
        FROM folders
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(&services.db_pool)
    .await
    .context("failed to fetch folders")?;

    println!("=== FOLDERS ===");
    if folders.is_empty() {
        println!("No folders found in database");
    } else {
        println!("Found {} folders:", folders.len());
        println!(
            "{:<5} {:<20} {:<10} {:<30} {:<10} {:<10} {:<10} {:<25} {:<25}",
            "ID",
            "Public ID",
            "User ID",
            "Name",
            "Color",
            "Parent ID",
            "Collapsed",
            "Created At",
            "Updated At"
        );
        println!("{}", "-".repeat(150));

        for folder in folders {
            let id: i64 = folder.get("id");
            let public_id: String = folder.get("public_id");
            let user_id: i64 = folder.get("user_id");
            let name: String = folder.get("name");
            let color: Option<String> = folder.get("color");
            let parent_id: Option<i64> = folder.get("parent_id");
            let collapsed: bool = folder.get("collapsed");
            let created_at: String = folder.get("created_at");
            let updated_at: String = folder.get("updated_at");

            println!(
                "{:<5} {:<20} {:<10} {:<30} {:<10} {:<10} {:<10} {:<25} {:<25}",
                id,
                public_id,
                user_id,
                name,
                color.as_deref().unwrap_or("NULL"),
                parent_id
                    .map(|id| id.to_string())
                    .unwrap_or("NULL".to_string()),
                collapsed,
                created_at,
                updated_at
            );
        }
    }

    println!("\n=== CHATS ===");
    // Dump chats
    let chats = sqlx::query(
        r#"
        SELECT id, public_id, user_id, folder_id, title, is_group, created_at, updated_at
        FROM chats
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(&services.db_pool)
    .await
    .context("failed to fetch chats")?;

    if chats.is_empty() {
        println!("No chats found in database");
    } else {
        println!("Found {} chats:", chats.len());
        println!(
            "{:<5} {:<20} {:<10} {:<10} {:<30} {:<10} {:<25} {:<25}",
            "ID",
            "Public ID",
            "User ID",
            "Folder ID",
            "Title",
            "Is Group",
            "Created At",
            "Updated At"
        );
        println!("{}", "-".repeat(160));

        for chat in chats {
            let id: i64 = chat.get("id");
            let public_id: String = chat.get("public_id");
            let user_id: i64 = chat.get("user_id");
            let folder_id: Option<i64> = chat.get("folder_id");
            let title: String = chat.get("title");
            let is_group: bool = chat.get("is_group");
            let created_at: String = chat.get("created_at");
            let updated_at: String = chat.get("updated_at");

            println!(
                "{:<5} {:<20} {:<10} {:<10} {:<30} {:<10} {:<25} {:<25}",
                id,
                public_id,
                user_id,
                folder_id
                    .map(|id| id.to_string())
                    .unwrap_or("NULL".to_string()),
                title,
                is_group,
                created_at,
                updated_at
            );
        }
    }

    // Dump chat members
    println!("\n=== CHAT MEMBERS ===");
    let chat_members = sqlx::query(
        r#"
        SELECT id, chat_id, user_id, role, joined_at
        FROM chat_members
        ORDER BY joined_at ASC
        "#,
    )
    .fetch_all(&services.db_pool)
    .await
    .context("failed to fetch chat members")?;

    if chat_members.is_empty() {
        println!("No chat members found in database");
    } else {
        println!("Found {} chat members:", chat_members.len());
        println!(
            "{:<5} {:<10} {:<10} {:<15} {:<25}",
            "ID", "Chat ID", "User ID", "Role", "Joined At"
        );
        println!("{}", "-".repeat(70));

        for member in chat_members {
            let id: i64 = member.get("id");
            let chat_id: i64 = member.get("chat_id");
            let user_id: i64 = member.get("user_id");
            let role: String = member.get("role");
            let joined_at: String = member.get("joined_at");

            println!(
                "{:<5} {:<10} {:<10} {:<15} {:<25}",
                id, chat_id, user_id, role, joined_at
            );
        }
    }

    // Dump messages
    println!("\n=== MESSAGES ===");
    let messages = sqlx::query(
        r#"
        SELECT id, public_id, chat_id, user_id, content, message_type, reply_to_id, created_at, updated_at
        FROM messages
        ORDER BY created_at ASC
        "#
    )
    .fetch_all(&services.db_pool)
    .await
    .context("failed to fetch messages")?;

    if messages.is_empty() {
        println!("No messages found in database");
    } else {
        println!("Found {} messages:", messages.len());
        println!(
            "{:<5} {:<20} {:<10} {:<10} {:<50} {:<15} {:<12} {:<25} {:<25}",
            "ID",
            "Public ID",
            "Chat ID",
            "User ID",
            "Content (truncated)",
            "Type",
            "Reply To",
            "Created At",
            "Updated At"
        );
        println!("{}", "-".repeat(200));

        for message in messages {
            let id: i64 = message.get("id");
            let public_id: String = message.get("public_id");
            let chat_id: i64 = message.get("chat_id");
            let user_id: i64 = message.get("user_id");
            let content: String = message.get("content");
            let message_type: String = message.get("message_type");
            let reply_to_id: Option<i64> = message.get("reply_to_id");
            let created_at: String = message.get("created_at");
            let updated_at: String = message.get("updated_at");

            let content_display = if content.len() > 47 {
                format!("{}...", &content[..44])
            } else {
                content
            };

            println!(
                "{:<5} {:<20} {:<10} {:<10} {:<50} {:<15} {:<12} {:<25} {:<25}",
                id,
                public_id,
                chat_id,
                user_id,
                content_display,
                message_type,
                reply_to_id
                    .map(|id| id.to_string())
                    .unwrap_or("NULL".to_string()),
                created_at,
                updated_at
            );
        }
    }

    Ok(())
}

async fn clear_data() -> anyhow::Result<()> {
    telemetry::init_tracing().context("failed to initialise tracing")?;

    info!("clearing all data from database");

    let config = load_config().context("failed to load configuration")?;

    let services = BackendServices::initialise(&config)
        .await
        .context("failed to initialise backend services")?;

    // Clear chats first (due to foreign key constraints)
    let chats_deleted = sqlx::query("DELETE FROM chats")
        .execute(&services.db_pool)
        .await
        .context("failed to delete chats")?;

    // Clear folders
    let folders_deleted = sqlx::query("DELETE FROM folders")
        .execute(&services.db_pool)
        .await
        .context("failed to delete folders")?;

    println!("Database cleared:");
    println!("- {} chats deleted", chats_deleted.rows_affected());
    println!("- {} folders deleted", folders_deleted.rows_affected());

    Ok(())
}

async fn seed_data() -> anyhow::Result<()> {
    telemetry::init_tracing().context("failed to initialise tracing")?;

    info!("seeding database with test data");

    let config = load_config().context("failed to load configuration")?;

    let services = BackendServices::initialise(&config)
        .await
        .context("failed to initialise backend services")?;

    // Insert test folders
    sqlx::query(
        r#"
        INSERT INTO folders (public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("test-folder-1")
    .bind(1i64)
    .bind("Work Projects")
    .bind("#ff6b6b")
    .bind(None::<i64>)
    .bind(false)
    .bind("2024-01-01T10:00:00Z")
    .bind("2024-01-01T10:00:00Z")
    .execute(&services.db_pool)
    .await
    .context("failed to insert test folder 1")?;

    sqlx::query(
        r#"
        INSERT INTO folders (public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("test-folder-2")
    .bind(1i64)
    .bind("Personal")
    .bind("#4ecdc4")
    .bind(None::<i64>)
    .bind(false)
    .bind("2024-01-01T11:00:00Z")
    .bind("2024-01-01T11:00:00Z")
    .execute(&services.db_pool)
    .await
    .context("failed to insert test folder 2")?;

    // Insert test chats (no folder relationships for simplicity)
    sqlx::query(
        r#"
        INSERT INTO chats (public_id, user_id, folder_id, title, messages, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("test-chat-1")
    .bind(1i64)
    .bind(None::<i64>) // no folder
    .bind("React Component Help")
    .bind(r#"[{"role": "user", "content": "How do I create a reusable React component?"}]"#)
    .bind("2024-01-01T12:00:00Z")
    .bind("2024-01-01T12:00:00Z")
    .execute(&services.db_pool)
    .await
    .context("failed to insert test chat 1")?;

    sqlx::query(
        r#"
        INSERT INTO chats (public_id, user_id, folder_id, title, messages, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("test-chat-2")
    .bind(1i64)
    .bind(None::<i64>) // no folder
    .bind("General Questions")
    .bind(r#"[{"role": "user", "content": "What's the best way to learn Rust?"}]"#)
    .bind("2024-01-01T13:00:00Z")
    .bind("2024-01-01T13:00:00Z")
    .execute(&services.db_pool)
    .await
    .context("failed to insert test chat 2")?;

    println!("Database seeded with test data:");
    println!("- 2 folders created");
    println!("- 2 chats created");
    println!("Run 'dump-data' to see the inserted data");

    Ok(())
}

async fn run_console() -> anyhow::Result<()> {
    telemetry::init_tracing().context("failed to initialise tracing")?;

    info!("starting interactive console");

    let config = load_config().context("failed to load configuration")?;

    let services = BackendServices::initialise(&config)
        .await
        .context("failed to initialise backend services")?;

    println!("Switchboard Interactive Console");
    println!("Type commands like '/help', '/folders', '/chats', '/clear', '/seed', '/quit'");
    println!("Use Ctrl+C or '/quit' to exit");
    println!("---");

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout())?;

        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break; // EOF
        }

        let command = line.trim();
        if command.is_empty() {
            continue;
        }

        match command {
            "/quit" | "/exit" | "/q" => {
                println!("Goodbye!");
                break;
            }
            "/help" | "/h" => {
                println!("Available commands:");
                println!("  /help, /h          - Show this help");
                println!("  /folders, /f       - List all folders");
                println!("  /chats, /c         - List all chats");
                println!("  /users, /u         - List all users");
                println!("  /clear, /cl        - Clear all data");
                println!("  /seed, /s          - Seed with test data");
                println!("  /dump, /d          - Dump all data");
                println!("  /quit, /exit, /q   - Exit console");
            }
            "/folders" | "/f" => {
                let folders = sqlx::query(
                    r#"
                    SELECT id, public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at
                    FROM folders
                    ORDER BY created_at ASC
                    "#
                )
                .fetch_all(&services.db_pool)
                .await
                .context("failed to fetch folders")?;

                if folders.is_empty() {
                    println!("No folders found");
                } else {
                    println!("Folders:");
                    for folder in folders {
                        let id: i64 = folder.get("id");
                        let _public_id: String = folder.get("public_id");
                        let name: String = folder.get("name");
                        let color: Option<String> = folder.get("color");
                        println!(
                            "  {}: {} ({})",
                            id,
                            name,
                            color.unwrap_or_else(|| "no color".to_string())
                        );
                    }
                }
            }
            "/chats" | "/c" => {
                let chats = sqlx::query(
                    r#"
                    SELECT id, public_id, user_id, folder_id, title, messages, created_at, updated_at
                    FROM chats
                    ORDER BY created_at ASC
                    "#
                )
                .fetch_all(&services.db_pool)
                .await
                .context("failed to fetch chats")?;

                if chats.is_empty() {
                    println!("No chats found");
                } else {
                    println!("Chats:");
                    for chat in chats {
                        let id: i64 = chat.get("id");
                        let title: String = chat.get("title");
                        let folder_id: Option<i64> = chat.get("folder_id");
                        println!(
                            "  {}: {} (folder: {})",
                            id,
                            title,
                            folder_id
                                .map(|id| id.to_string())
                                .unwrap_or_else(|| "none".to_string())
                        );
                    }
                }
            }
            "/users" | "/u" => {
                // For now, just show a placeholder since we don't have a users table
                println!("Users table not implemented yet");
                println!("Test user: ID 1, test-user@example.com");
            }
            "/clear" | "/cl" => {
                let chats_deleted = sqlx::query("DELETE FROM chats")
                    .execute(&services.db_pool)
                    .await
                    .context("failed to delete chats")?;

                let folders_deleted = sqlx::query("DELETE FROM folders")
                    .execute(&services.db_pool)
                    .await
                    .context("failed to delete folders")?;

                println!(
                    "Cleared {} chats and {} folders",
                    chats_deleted.rows_affected(),
                    folders_deleted.rows_affected()
                );
            }
            "/seed" | "/s" => {
                // Insert test folders
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO folders (public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind("console-folder-1")
                .bind(1i64)
                .bind("Console Folder")
                .bind("#ff6b6b")
                .bind(None::<i64>)
                .bind(false)
                .bind(sqlx::types::chrono::Utc::now().to_rfc3339())
                .bind(sqlx::types::chrono::Utc::now().to_rfc3339())
                .execute(&services.db_pool)
                .await
                .context("failed to insert test folder")?;

                // Insert test chat
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO chats (public_id, user_id, folder_id, title, messages, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind("console-chat-1")
                .bind(1i64)
                .bind(None::<i64>)
                .bind("Console Chat")
                .bind(r#"[{"role": "user", "content": "Hello from console!"}]"#)
                .bind(sqlx::types::chrono::Utc::now().to_rfc3339())
                .bind(sqlx::types::chrono::Utc::now().to_rfc3339())
                .execute(&services.db_pool)
                .await
                .context("failed to insert test chat")?;

                println!("Seeded test data (using OR IGNORE to avoid duplicates)");
            }
            "/dump" | "/d" => {
                dump_data().await?;
            }
            _ => {
                println!("Unknown command: {}", command);
                println!("Type '/help' for available commands");
            }
        }
    }

    Ok(())
}
