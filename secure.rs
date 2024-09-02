use anchor_lang::prelude::*;

declare_id!("6L2Rzxs71PiAxUmUxaNTT2Q3mnQjiJ8DwWiV1UxKa7Ph");

const MAX_NAME_LENGTH: usize = 10;
const USER_SEED: &[u8] = b"user";

#[program]
pub mod secure_user_points_system {
    use super::*;

    pub fn initialize(ctx: Context<CreateUser>, id: u32, name: String) -> Result<()> {
        require!(name.len() <= MAX_NAME_LENGTH, MyError::NameTooLong);

        let user = &mut ctx.accounts.user;
        user.id = id;
        user.owner = ctx.accounts.signer.key();
        user.name = name;
        user.points = 1000;

        msg!("Created new user with 1000 points and id: {}", id);
        Ok(())
    }

    pub fn transfer_points(ctx: Context<TransferPoints>, amount: u16) -> Result<()> {
        let sender = &mut ctx.accounts.sender;
        let receiver = &mut ctx.accounts.receiver;

        require!(amount > 0, MyError::InvalidTransferAmount);
        require!(sender.key() != receiver.key(), MyError::IdenticalAccounts);
        require!(receiver.owner != Pubkey::default(), MyError::AccountDoesNotExist);

        sender.points = sender.points.checked_sub(amount).ok_or(MyError::Underflow)?;
        receiver.points = receiver.points.checked_add(amount).ok_or(MyError::Overflow)?;

        msg!("Transferred {} points from user {} to user {}", amount, sender.id, receiver.id);
        Ok(())
    }

    pub fn remove_user(ctx: Context<RemoveUser>) -> Result<()> {
        let user = &ctx.accounts.user;
        require!(user.owner != Pubkey::default(), MyError::AccountDoesNotExist);
        msg!("Account closed for user with id: {}", user.id);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(id: u32, name: String)]
pub struct CreateUser<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + 4 + 32 + (4 + MAX_NAME_LENGTH) + 2,
        seeds = [USER_SEED, id.to_le_bytes().as_ref()],
        bump
    )]
    pub user: Account<'info, User>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferPoints<'info> {
    #[account(
        mut,
        seeds = [USER_SEED, sender.id.to_le_bytes().as_ref()],
        bump,
        constraint = signer.key() == sender.owner @ MyError::Unauthorized
    )]
    pub sender: Account<'info, User>,

    #[account(
        mut,
        seeds = [USER_SEED, receiver.id.to_le_bytes().as_ref()],
        bump
    )]
    pub receiver: Account<'info, User>,

    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveUser<'info> {
    #[account(
        mut,
        close = signer,
        seeds = [USER_SEED, user.id.to_le_bytes().as_ref()],
        bump,
        constraint = signer.key() == user.owner @ MyError::Unauthorized
    )]
    pub user: Account<'info, User>,

    #[account(mut)]
    pub signer: Signer<'info>,
}

#[account]
#[derive(Default)]
pub struct User {
    pub id: u32,
    pub owner: Pubkey,
    pub name: String,
    pub points: u16,
}

#[error_code]
pub enum MyError {
    #[msg("Not enough points to transfer")]
    NotEnoughPoints,
    #[msg("Cannot transfer zero or negative points")]
    InvalidTransferAmount,
    #[msg("Arithmetic overflow occurred")]
    Overflow,
    #[msg("Arithmetic underflow occurred")]
    Underflow,
    #[msg("Name is too long")]
    NameTooLong,
    #[msg("User account does not exist")]
    AccountDoesNotExist,
    #[msg("Sender and receiver accounts are identical")]
    IdenticalAccounts,
    #[msg("Unauthorized operation")]
    Unauthorized,
}