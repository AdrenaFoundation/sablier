use anchor_lang::prelude::*;

pub trait AccountInfoExt<'info>: AsRef<AccountInfo<'info>> {
    fn realloc(&self, new_len: usize, zero_init: bool) -> Result<()> {
        self.as_ref().realloc(new_len, zero_init)?;
        Ok(())
    }

    fn data_len(&self) -> usize {
        self.as_ref().data_len()
    }
}

impl<'info, T: AsRef<AccountInfo<'info>>> AccountInfoExt<'info> for T {}
