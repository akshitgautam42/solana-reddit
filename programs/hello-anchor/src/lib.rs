use anchor_lang::prelude::*;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("11111111111111111111111111111111");

const USERNAMELEN: usize = 100;
const THREAD_CONTENT_LEN: usize = 400;
const THREAD_TITLE_LEN: usize = 200;
const COMMENT_LEN: usize = 400;

#[program]
mod hello_anchor {
    use super::*;
    pub fn create_user(ctx: Context<Initialize>, username: String) -> Result<()> {
        let user = &mut ctx.accounts.user;
        user.authority = ctx.accounts.authority;
        user.username = username;
        msg!("Changed data to: {}!", data); // Message will show up in the tx logs
        Ok(())
    }

    pub fn create_feed(ctx: Context<CreateFeed>) -> Result<()> {
        let feed = &mut ctx.accounts.feed;
        feed.thread_count = 0;
        Ok(())
    }

    pub fn create_thread(ctx: Context<CreateThread>, content: String, title: String) -> Result<()> {
        let feed = &mut ctx.accounts.feed;

        feed.thread_count += 1;

        let user = &mut ctx.accounts.user;
        let thread = &mut ctx.accounts.thread;

        thread.authority = ctx.accounts.user.key();
        thread.content = content;
        thread.title = title;
        thread.timestamp = ctx.accounts.clock.unix_timestamp;
        thread.owner_username = user.username.clone();
        thread.vote = 0;

        Ok(())
    }

    pub fn create_thread_upvote(
        ctx: Context<CreateThreadUpvote>,
        thread_id: u64,
        is_up: bool,
    ) -> Result<()> {
        let thread = &mut ctx.accounts.thread;
        thread.vote += if is_up { 1 } else { -1 };

        let upvote = &mut ctx.accounts.thread_vote;
        upvote.vote = if is_up { 1 } else { -1 };
        upvote.authority = ctx.accounts.authority.key();
        upvote.thread_id = thread_id;
        Ok(())
    }

    pub fn create_comment(
        ctx: Context<CreateComment>,
        _thread_id: u64,
        content: String,
        is_root: bool,
        parent_id: u64,
    ) -> Result<()> {
        let thread = &mut ctx.accounts.thread;
        thread.comment_count += 1;

        let comment = &mut ctx.accounts.comment;
        comment.authority = ctx.accounts.authority.key();
        comment.parent_comment = parent_id;
        comment.content = content;
        comment.vote = 0;
        comment.is_root = is_root;
        Ok(())
    }

    pub fn create_comment_upvote(
        ctx: Context<CreateCommentUpvote>,
        _thread_id: u64,
        _comment_id: u64,
        is_up: bool,
    ) -> Result<()> {
        let comment = &mut ctx.accounts.comment;
        comment.vote += if is_up { 1 } else { -1 };

        let comment_vote = &mut ctx.accounts.comment_vote;
        comment_vote.vote = if is_up { 1 } else { -1 };
        comment_vote.authority = ctx.accounts.authority.key();

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateUSer<'info> {
    // We must specify the space in order to initialize an account.
    // First 8 bytes are default account discriminator,
    // next 8 bytes come from NewAccount.data being type u64.
    // (u64 = 64 bits unsigned integer = 8 bytes)
    #[account(init, payer = authority,seeds=[b"user".as_ref(),authority.key().as_ref()], space = size_of::<UserAccount>+4*USERNAMELEN+8 + 8)]
    pub user: Account<'info, UserAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateFeed<'info> {
    #[account(
        init,
        seeds=[b"feed".as_ref()],
        bump,
        payer=authority,
        space=size_of::<FeedAccount>()+8
    )]
    pub feed: Account<'info, FeedAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateThread<'info> {
    #[account(
        mut,
        seeds = [b"thread".as_ref(),authority.key().as_ref()],
        bump
    )]
    pub user: Account<'info, UserAccount>,
    #[account(
        seeds=[b"feed".as_ref()],
        bump,
    )]
    pub feed: Account<'info, FeedAccount>,

    #[account(
        init,
        seeds=[b"thread".as_ref(),feed.thread_count.to_be_bytes().as_ref()],
        bump,
        payer=authority,
        space = size_of::<ThreadAccount>()+4*THREAD_CONTENT_LEN+4*THREAD_TITLE_LEN+8,
    )]
    pub thread: Account<'info, ThreadAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instructions(thread_id:u64)]
pub struct CreateThreadUpvote<'info> {
    #[account(mut, seeds=[b"thread".as_ref(),thread_id.to_be_bytes().as_ref()],bump)]
    pub thread: Account<'info, ThreadAccount>,

    #[account(
        init,
        seeds = [b"thread_vote".as_ref(),authority.key().as_ref(),thread_id.to_be_bytes().as_ref()],
        bump,
        payer=authority,
        space=size_of::<ThreadUpvoteAccount>()+8
    )]
    pub thread_vote: Account<'info, ThreadUpvoteAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instructions(thread_id:u64)]
pub struct CreateComment<'info> {
    #[account(mut,seeds=[b"thread".as_ref(),thread_id.to_be_bytes().as_ref()],bump)]
    pub thread: Account<'info, ThreadAccount>,

    #[account(
        init,
        seeds = [b"comment".as_ref(),thread_id.to_be_bytes().as_ref(),thread.comment_count.to_be_bytes().as_ref()],
        bump,
        payer=authority,
        space = size_of::<CommentAccount>()+4*COMMENT_LEN+8
    )]
    pub comment: Account<'info, CommentAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instructions(thread_id:u64,comment_id:u64)]
pub struct CreateCommentUpvote<'info> {
    #[account(
        mut,
        seeds = [b"comment".as_ref(),thread_id.to_be_bytes().as_ref(),comment_id.to_be_bytes().as_ref()],
        bump
    )]
    pub comment: Account<'info, CommentAccount>,
    #[account(
        init,
        seeds=[b"comment_vote".as_ref(),
        thread_id.to_be_bytes().as_ref(),
        comment_id.to_be_bytes().as_ref(),
        authority.key().as_ref()],
        bump,
        payer=authority,
        space = size_of::<CommentVoteAccount>() + 8
    )]
    pub comment_vote: Account<'info, CommentVoteAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct UserAccount {
    pub authority: Pubkey,
    pub username: String,
}

#[account]
pub struct FeedAccount {
    pub thread_count: u64,
}

#[account]
pub struct ThreadAccount {
    pub authority: Pubkey,

    pub title: String,

    pub content: String,

    pub timestamp: i64,

    pub vote: i64,

    pub comment_count: u64,

    pub owner_username: String,
}

#[account]
pub struct ThreadUpvoteAccount {
    pub authority: Pubkey,
    pub vote: i8,
    pub thread_id: u64,
}

#[account]
pub struct CommentAccount {
    pub authority: Pubkey,
    pub content: String,
    pub is_root: bool,
    pub parent_comment: u64,
    pub vote: i8,
}

#[account]
pub struct CommentVoteAccount {
    pub authority: Pubkey,
    pub vote: i8,
}
