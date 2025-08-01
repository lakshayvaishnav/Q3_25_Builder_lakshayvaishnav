use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ close_account, transfer, CloseAccount, Mint, Token, TokenAccount, Transfer },
};

use crate::{Escrow, LazyEscrow};

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub token_a: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_a,
        associated_token::authority = maker,
    )]
    pub ata_maker_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_a,
        associated_token::authority = escrow,
    )]
    pub vault_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        close = maker,
        seeds = [b"escrow", maker.key().as_ref()],
        bump = *(escrow.load_bump()?),
        has_one = maker,
        has_one = token_a
    )]
    pub escrow: LazyAccount<'info, Escrow>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund_to_maker(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault_token_a.to_account_info(),
            to: self.ata_maker_token_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let signer_seeds: [&[&[u8]]; 1] = [
            &[b"escrow", self.maker.to_account_info().key.as_ref(), &[*self.escrow.load_bump()?]],
        ];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

        let amount_a = self.escrow.load_amount_a()?;

        transfer(cpi_ctx, *amount_a)?;

        Ok(())
    }
    
    pub fn close_vault(&mut self) -> Result<()> {
        let close_accounts = CloseAccount {
            account: self.vault_token_a.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let signer_seeds: [&[&[u8]]; 1] = [
            &[b"escrow", self.maker.to_account_info().key.as_ref(), &[*self.escrow.load_bump()?]],
        ];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close_accounts,
            &signer_seeds
        );

        close_account(ctx)
    }
}
