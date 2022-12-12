use solana_program::{
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
    account_info::{next_account_info, AccountInfo},
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
    program::{invoke_signed},
    borsh::try_from_slice_unchecked,
    program_error::ProgramError,
};
use std::convert::TryInto;
use borsh::BorshSerialize;
use crate::instruction::MovieInstruction;
use crate::state::MovieAccountState;
use crate::error::ReviewError;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
  ) -> ProgramResult {
    let instruction = MovieInstruction::unpack(instruction_data)?;
    match instruction {
      MovieInstruction::AddMovieReview { title, rating, description } => {
        add_movie_review(program_id, accounts, title, rating, description)
      }
      MovieInstruction::UpdateMovieReview { title, rating, description } => {
        update_movie_review(program_id, accounts, title, rating, description)
      }
    }
  }

  pub fn add_movie_review(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    title: String,
    rating: u8,
    description: String
  ) -> ProgramResult {
    msg!("Adding movie review...");
    msg!("Title: {}", title);
    msg!("Rating: {}", rating);
    msg!("Description: {}", description);

    if rating > 5 || rating <1 {
        msg!("Rating must be between 1 and 5");
        return Err(ReviewError::InvalidRating.into())
    }

    let total_len: usize = 1 + 1 + (4 + title.len()) + (4 + description.len());
    if total_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(ReviewError::InvalidDataLength.into())
    }
    
    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let user_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature.into())
    }
        
    let (pda, bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref(), title.as_bytes().as_ref(),], program_id);
    if pda != *user_account.key {
        msg!("Invalid seeds for PDA");
        return Err(ProgramError::InvalidArgument.into())
    }

    let account_len = 1000;

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    invoke_signed(
      &system_instruction::create_account(
        initializer.key,
        user_account.key,
        rent_lamports,
        account_len.try_into().unwrap(),
        program_id,
      ),
      &[initializer.clone(), user_account.clone(), system_program.clone()],
      &[&[initializer.key.as_ref(), title.as_bytes().as_ref(), &[bump_seed]]],
    )?;

    msg!("PDA created: {}", pda);

    msg!("unpacking state account");
    let mut account_data = try_from_slice_unchecked::<MovieAccountState>(&user_account.data.borrow()).unwrap();
    msg!("borrowed account data");

    account_data.title = title;
    account_data.rating = rating;
    account_data.description = description;
    account_data.is_initialized = true;

    msg!("serializing account");
    account_data.serialize(&mut &mut user_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    Ok(())
  }

  pub fn update_movie_review(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _title: String,
    rating: u8,
    description: String
  ) -> ProgramResult {
    msg!("Update movie review...");
    msg!("Rating: {}", rating);
    msg!("Description: {}", description);

    if rating > 5 || rating <1 {
        msg!("Rating must be between 1 and 5");
        return Err(ReviewError::InvalidRating.into())
    }

    let total_len: usize = 1 + 1 + (4 + _title.len()) + (4 + description.len());
    if total_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(ReviewError::InvalidDataLength.into())
    }

    // test
    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let user_account = next_account_info(account_info_iter)?;
    if user_account.owner != program_id {
        return Err(ProgramError::IllegalOwner)
    }      

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature.into())
    }
        
    msg!("unpacking state account");
    let mut account_data = try_from_slice_unchecked::<MovieAccountState>(&user_account.data.borrow()).unwrap();
    msg!("borrowed account data");

    let (pda, bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref(), account_data.title.as_bytes().as_ref(),], program_id);
    if pda != *user_account.key {
        msg!("Invalid seeds for PDA");
        return Err(ProgramError::InvalidArgument.into())
    }

    if !account_data.is_initialized {
        msg!("Account is not initialized");
        return Err(ReviewError::UninitializedAccount.into());
    }      

    // ??????  
    let total_len: usize = 1 + 1 + (4 + account_data.title.len()) + (4 + description.len());
    if total_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(ReviewError::InvalidDataLength.into())
    }
      
    account_data.rating = rating;
    account_data.description = description;

    msg!("serializing account");
    account_data.serialize(&mut &mut user_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    Ok(())
  }