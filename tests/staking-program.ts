import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StakingProgram } from "../target/types/staking_program";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";

describe("staking-program", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = provider.wallet as anchor.Wallet;
  // const mintKeyPair = Keypair.generate();
  const mintKeyPair = Keypair.fromSecretKey(new Uint8Array([
    131,  48, 125,  29,   5,  35, 155,  59, 187, 151, 250,
    178,  47, 143,   1, 224, 236, 199, 244, 200, 216, 254,
    205, 248,  25, 101,   5, 163,  21, 218, 219, 252, 218,
     22,  91,  53,  91,  47,  33,  41,  85,  87, 244,  76,
    234,  35, 135, 114, 204,  25, 228,   7,  57, 175, 191,
    153,  60, 254,  63, 221, 119,  59,  99, 101
  ])); 
  console.log(mintKeyPair);

  const connection = new Connection("http://127.0.0.1:8899", "confirmed");

  const program = anchor.workspace.StakingProgram as Program<StakingProgram>;

  async function createMintToken() {
    const mint = await createMint(
      connection, 
      payer.payer,
      payer.publicKey,
      payer.publicKey,
      9,
      mintKeyPair
    )

    console.log(mint)
  }

  it("Is initialized!", async () => {
    // Add your test here.
    // await createMintToken();

    let [vaultAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault")],
      program.programId
    )

    const tx = await program.methods.initialize()
      .accounts({
        signer: payer.publicKey,
        tokenVaultAccount: vaultAccount, 
        mint: mintKeyPair.publicKey
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });

  it("Stake!", async () => {

    let userTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection, 
      payer.payer,
      mintKeyPair.publicKey,
      payer.publicKey
    );

    await mintTo(
      connection, 
      payer.payer,
      mintKeyPair.publicKey,
      userTokenAccount.address,
      payer.payer,
      1e11
    );

    let [stakeInfo] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake_info"), payer.publicKey.toBuffer()],
      program.programId
    );

    let [stakeAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("token"), payer.publicKey.toBuffer()],
      program.programId
    );

    await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintKeyPair.publicKey,
      payer.publicKey
    );


    const tx = await program.methods
      .stake(new anchor.BN(1), new anchor.BN(1))
      .signers([payer.payer])
      .accounts({
        stakeInfoAccount: stakeInfo,
        stakeAccount: stakeAccount,
        userTokenAccount: userTokenAccount.address,
        mint: mintKeyPair.publicKey,
        signer: payer.publicKey
      })
      .rpc();

    console.log("Your transaction signature", tx);

  });

  it("DeStake!", async () => {

    let userTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection, 
      payer.payer,
      mintKeyPair.publicKey,
      payer.publicKey
    );

    let [stakeInfo] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake_info"), payer.publicKey.toBuffer()],
      program.programId
    );

    let [stakeAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("token"), payer.publicKey.toBuffer()],
      program.programId
    );

    let [vaultAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault")],
      program.programId
    )

    await mintTo(
      connection,
      payer.payer,
      mintKeyPair.publicKey,
      vaultAccount,
      payer.payer,
      1e21
    );

    const tx = await program.methods
      .destake()
      .signers([payer.payer])
      .accounts({
        stakeAccount: stakeAccount,
        stakeInfoAccount: stakeInfo,
        userTokenAccount: userTokenAccount.address,
        tokenVaultAccount: vaultAccount,
        signer: payer.publicKey,
        mint: mintKeyPair.publicKey
      })
      .rpc();

    console.log("Your transaction signature", tx);

  });

});
