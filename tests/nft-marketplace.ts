import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { NftMarketplace } from '../target/types/nft_marketplace';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from '@solana/spl-token';

describe('nft-marketplace', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const connection = provider.connection;

  const program = anchor.workspace.NftMarketplace as Program<NftMarketplace>;

  let maker = anchor.web3.Keypair.generate();
  let taker = anchor.web3.Keypair.generate();

  let mint;
  let maker_ata;

  before(async () => {
    //airdrop the accounts
    const makerAirdrop = await connection.requestAirdrop(
      maker.publicKey,
      7 * LAMPORTS_PER_SOL
    );
    const latestBlockhash = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature: makerAirdrop,
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    });

    const takerAirdrop = await connection.requestAirdrop(
      taker.publicKey,
      7 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction({
      signature: takerAirdrop,
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    });

    //create mint account
    mint = await createMint(connection, maker, maker.publicKey, null, 0);

    //create maker ATA
    maker_ata = await getOrCreateAssociatedTokenAccount(
      connection,
      maker,
      mint,
      maker.publicKey
    ); //owner not off curve

    //MintTo nft maker ATA
    mintTo(connection, maker, mint, maker_ata.address, maker, 1);
  });

  it('Is initialized!', async () => {
    // Add your test here.
  });
});
