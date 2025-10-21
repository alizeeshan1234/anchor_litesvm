use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("2vsYWAyAJb85kLDeufufpTCojtVLWCaUCUCUfiq1mgpG");

#[program]
pub mod litesvm_exp {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn initialize_counter(ctx: Context<Counter>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count = 0;
        msg!("Counter Account initialized with count: {}", counter.count);
        Ok(())
    }

    pub fn increment_counter(ctx: Context<CounterOperation>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count += 1;
        msg!("Counter incremented to: {}", counter.count);
        Ok(())
    }

    pub fn decrement_counter(ctx: Context<CounterOperation>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count -= 1;
        msg!("Counter decremented to: {}", counter.count);
        Ok(())
    }

    pub fn init_token_account(ctx: Context<InitTokenAccount>) -> Result<()> {
        msg!("Token account initialized for: {:?}", ctx.accounts.signer.key());
        Ok(())
    }

    pub fn transfer_tokens(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        let cpi_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.from_token_account.to_account_info(),
            to: ctx.accounts.to_token_account.to_account_info(),
            authority: ctx.accounts.from.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        anchor_spl::token::transfer(cpi_ctx, amount)?;
        msg!("Transferred {} tokens", amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct Counter<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = 8 + 8,
        seeds = [b"counter"],
        bump
    )]
    pub counter: Account<'info, CounterAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CounterOperation<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"counter"],
        bump
    )]
    pub counter: Account<'info, CounterAccount>,
}

#[derive(Accounts)]
pub struct InitTokenAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer,
    )]
    pub token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub from: Signer<'info>,

    pub to: UncheckedAccount<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = from,
    )]
    pub from_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = from,
        associated_token::mint = mint,
        associated_token::authority = to,
    )]
    pub to_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[account]
pub struct CounterAccount {
    pub count: u64
}


#[cfg(test)]
mod testing {
    use litesvm::LiteSVM;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };
    use anchor_lang::{InstructionData, AccountDeserialize}; 

    use litesvm_token::{
        spl_token::{self, native_mint::DECIMALS, state::GenericTokenAccount},
        CreateAssociatedTokenAccount, CreateMint, MintTo,
    };

    use spl_token::solana_program::program_pack::Pack;

    #[test]
    fn initialize_counter() {
        let mut svm = LiteSVM::new();
        let program_id = crate::ID;
        let program_bytes = include_bytes!("../../../target/deploy/litesvm_exp.so");
        svm.add_program(program_id, program_bytes);
        println!("Program ID: {}", program_id);

        let signer = Keypair::new();
        svm.airdrop(&signer.pubkey(), 10_000_000_000).unwrap(); 

        let (counter_pda, _bump) = Pubkey::find_program_address(&[b"counter"], &program_id);

        let instruction_data = crate::instruction::InitializeCounter {}.data();

        let initialize_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(signer.pubkey(), true),
                AccountMeta::new(counter_pda, false),
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            ],
            data: instruction_data,
        };

        let tx = Transaction::new_signed_with_payer(
            &[initialize_instruction],
            Some(&signer.pubkey()),
            &[&signer],
            svm.latest_blockhash(),
        );

        svm.send_transaction(tx).unwrap();

        println!("Counter initialized successfully!");

        let counter_account = svm.get_account(&counter_pda).unwrap();
        let counter_data = crate::CounterAccount::try_deserialize(
            &mut counter_account.data.as_slice()
        ).unwrap();
        
        println!("Counter PDA: {}", counter_pda);
        println!("Counter value: {}", counter_data.count);
        assert_eq!(counter_data.count, 0);
    }

    #[test]
    fn increment_counter() {
        let mut svm = LiteSVM::new();
        let program_id = crate::ID;
        let program_bytes = include_bytes!("../../../target/deploy/litesvm_exp.so");
        svm.add_program(program_id, program_bytes);

        let signer = Keypair::new();
        svm.airdrop(&signer.pubkey(), 10_000_000_000).unwrap(); 

        let (counter_pda, _bump) = Pubkey::find_program_address(&[b"counter"], &program_id);

        let instruction_data = crate::instruction::InitializeCounter {}.data();
        let initialize_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(signer.pubkey(), true),
                AccountMeta::new(counter_pda, false),
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            ],
            data: instruction_data,
        };

        let tx = Transaction::new_signed_with_payer(
            &[initialize_instruction],
            Some(&signer.pubkey()),
            &[&signer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        let counter_account = svm.get_account(&counter_pda).unwrap();
        let counter_data = crate::CounterAccount::try_deserialize(
            &mut counter_account.data.as_slice()
        ).unwrap();
        println!("Initial counter: {}", counter_data.count);
        assert_eq!(counter_data.count, 0);

        let instruction_data = crate::instruction::IncrementCounter {}.data();
        let increment_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(signer.pubkey(), true),
                AccountMeta::new(counter_pda, false),
            ],
            data: instruction_data,
        };

        let tx = Transaction::new_signed_with_payer(
            &[increment_instruction],
            Some(&signer.pubkey()),
            &[&signer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        let counter_account = svm.get_account(&counter_pda).unwrap();
        let counter_data = crate::CounterAccount::try_deserialize(
            &mut counter_account.data.as_slice()
        ).unwrap();
        println!("After increment: {}", counter_data.count);
        assert_eq!(counter_data.count, 1);
    }

    #[test]
    fn decrement_counter() {
        let mut svm = LiteSVM::new();
        let program_id = crate::ID;
        let program_bytes = include_bytes!("../../../target/deploy/litesvm_exp.so");
        svm.add_program(program_id, program_bytes);

        let signer = Keypair::new();
        svm.airdrop(&signer.pubkey(), 10_000_000_000).unwrap(); 

        let (counter_pda, _bump) = Pubkey::find_program_address(&[b"counter"], &program_id);

        let instruction_data = crate::instruction::InitializeCounter {}.data();
        let initialize_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(signer.pubkey(), true),
                AccountMeta::new(counter_pda, false),
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            ],
            data: instruction_data,
        };

        let tx = Transaction::new_signed_with_payer(
            &[initialize_instruction],
            Some(&signer.pubkey()),
            &[&signer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        let counter_account = svm.get_account(&counter_pda).unwrap();
        let counter_data = crate::CounterAccount::try_deserialize(
            &mut counter_account.data.as_slice()
        ).unwrap();
        println!("Initial counter: {}", counter_data.count);
        assert_eq!(counter_data.count, 0);

        let instruction_data = crate::instruction::IncrementCounter {}.data();
        let increment_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(signer.pubkey(), true),
                AccountMeta::new(counter_pda, false),
            ],
            data: instruction_data,
        };

        let tx = Transaction::new_signed_with_payer(
            &[increment_instruction],
            Some(&signer.pubkey()),
            &[&signer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        let counter_account = svm.get_account(&counter_pda).unwrap();
        let counter_data = crate::CounterAccount::try_deserialize(
            &mut counter_account.data.as_slice()
        ).unwrap();
        println!("After increment: {}", counter_data.count);

        let instruction_data = crate::instruction::DecrementCounter {}.data();
        let decrement_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(signer.pubkey(), true),
                AccountMeta::new(counter_pda, false),
            ],
            data: instruction_data,
        };

        let tx = Transaction::new_signed_with_payer(
            &[decrement_instruction],
            Some(&signer.pubkey()),
            &[&signer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        let counter_account = svm.get_account(&counter_pda).unwrap();
        let counter_data = crate::CounterAccount::try_deserialize(
            &mut counter_account.data.as_slice()
        ).unwrap();
        println!("After decrement: {}", counter_data.count);
        assert_eq!(counter_data.count, 0);
    }

    #[test]
    fn initialize_token_account() {
        let mut svm = LiteSVM::new();
        let program_id = crate::ID;
        let program_bytes = include_bytes!("../../../target/deploy/litesvm_exp.so");
        svm.add_program(program_id, program_bytes);
        println!("Program ID: {}", program_id);

        let signer = Keypair::new();
        svm.airdrop(&signer.pubkey(), 10_000_000_000).unwrap(); 

        let mint = CreateMint::new(&mut svm, &signer)
            .authority(&signer.pubkey())
            .decimals(DECIMALS)
            .send()
            .unwrap();

        let signer_ata = CreateAssociatedTokenAccount::new(&mut svm, &signer, &mint)
            .owner(&signer.pubkey())
            .send()
            .unwrap();

        MintTo::new(&mut svm, &signer, &mint, &signer_ata, 1_000_000_000) 
            .send()
            .unwrap();

        let instruction_data = crate::instruction::InitTokenAccount {}.data();

        let initialize_token_account_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(signer.pubkey(), true), 
                AccountMeta::new(mint, false),
                AccountMeta::new(signer_ata, false),  
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            ],
            data: instruction_data,
        };

        let tx = Transaction::new_signed_with_payer(
            &[initialize_token_account_instruction],
            Some(&signer.pubkey()),
            &[&signer],
            svm.latest_blockhash(),
        );

        svm.send_transaction(tx).unwrap();

        println!("Token account initialized successfully!");

        let token_account = svm.get_account(&signer_ata).unwrap();
        println!("Token Account: {}", signer_ata);
    }

    #[test]
    fn transfer_tokens() {
        let mut svm = LiteSVM::new();
        let program_id = crate::ID;
        let program_bytes = include_bytes!("../../../target/deploy/litesvm_exp.so");
        svm.add_program(program_id, program_bytes);
        println!("Program ID: {}", program_id);

        let sender = Keypair::new();
        svm.airdrop(&sender.pubkey(), 10_000_000_000).unwrap(); 

        let receiver = Keypair::new();
        svm.airdrop(&receiver.pubkey(), 10_000_000_000).unwrap(); 

        let mint = CreateMint::new(&mut svm, &sender)
            .authority(&sender.pubkey())
            .decimals(DECIMALS)
            .send()
            .unwrap();

        let sender_ata = CreateAssociatedTokenAccount::new(&mut svm, &sender, &mint)
            .owner(&sender.pubkey())
            .send()
            .unwrap();

        let receiver_ata = CreateAssociatedTokenAccount::new(&mut svm, &receiver, &mint)
            .owner(&receiver.pubkey())
            .send()
            .unwrap();

        MintTo::new(&mut svm, &sender, &mint, &sender_ata, 1_000_000_000) 
            .send()
            .unwrap();

        let instruction_data = crate::instruction::TransferTokens { amount: 500_000_000 }.data();

        let transfer_tokens_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(sender.pubkey(), true), 
                AccountMeta::new(receiver.pubkey(), false), 
                AccountMeta::new(mint, false),
                AccountMeta::new(sender_ata, false),  
                AccountMeta::new(receiver_ata, false),  
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            ],
            data: instruction_data,
        };

        let tx = Transaction::new_signed_with_payer(
            &[transfer_tokens_instruction],
            Some(&sender.pubkey()),
            &[&sender],
            svm.latest_blockhash(),
        );

        svm.send_transaction(tx).unwrap();

        println!("Tokens transferred successfully!");
    
        let sender_account_data = svm.get_account(&sender_ata).unwrap();
        let receiver_account_data = svm.get_account(&receiver_ata).unwrap();

        let sender_token_account = spl_token::state::Account::unpack(&sender_account_data.data).unwrap();
        let receiver_token_account = spl_token::state::Account::unpack(&receiver_account_data.data).unwrap();

        println!("Sender balance: {}", sender_token_account.amount);
        println!("Receiver balance: {}", receiver_token_account.amount);

        assert_eq!(sender_token_account.amount, 500_000_000); 
        assert_eq!(receiver_token_account.amount, 500_000_000); 
    }
}