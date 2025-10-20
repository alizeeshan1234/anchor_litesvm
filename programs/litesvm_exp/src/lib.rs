use anchor_lang::prelude::*;

declare_id!("4HADsmeSsErmhVZdvYbLTmF5iBuqo1fXXwSHL2apXn1x");

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
}