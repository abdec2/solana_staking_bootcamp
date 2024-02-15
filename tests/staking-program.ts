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
  const mintKeyPair = Keypair.generate();
  // const mintKeyPair = Keypair.fromSecretKey(new Uint8Array([
  //   254,  95, 193,   6, 151, 215,  52, 104,  31, 228,  69,
  //   212, 147, 249, 146, 115, 117,  84,  23, 150, 127, 178,
  //    32, 103, 151,  31,  89, 178,   6, 135,  15,  13, 192,
  //   201,  62, 254,  43,  70, 125, 121,   1, 115, 172, 119,
  //   155, 198, 118, 106, 254,  20, 205,  44, 172,  68, 230,
  //   145, 111, 198, 124, 133,  96, 237, 175,  77
  // ])); 
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

  // it("Is initialized!", async () => {
  //   // Add your test here.
  //   // await createMintToken();

  //   let [vaultAccount] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("vault")],
  //     program.programId
  //   )

  //   const tx = await program.methods.initialize()
  //     .accounts({
  //       signer: payer.publicKey,
  //       tokenVaultAccount: vaultAccount, 
  //       mint: mintKeyPair.publicKey
  //     })
  //     .rpc();
  //   console.log("Your transaction signature", tx);
  // });

  // it("Stake!", async () => {

  //   let userTokenAccount = await getOrCreateAssociatedTokenAccount(
  //     connection, 
  //     payer.payer,
  //     mintKeyPair.publicKey,
  //     payer.publicKey
  //   );

  //   await mintTo(
  //     connection, 
  //     payer.payer,
  //     mintKeyPair.publicKey,
  //     userTokenAccount.address,
  //     payer.payer,
  //     1e11
  //   );

  //   let [stakeInfo] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("stake_info"), payer.publicKey.toBuffer()],
  //     program.programId
  //   );

  //   let [stakeAccount] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("token"), payer.publicKey.toBuffer()],
  //     program.programId
  //   );

  //   await getOrCreateAssociatedTokenAccount(
  //     connection,
  //     payer.payer,
  //     mintKeyPair.publicKey,
  //     payer.publicKey
  //   );


  //   const tx = await program.methods
  //     .stake(new anchor.BN(1))
  //     .signers([payer.payer])
  //     .accounts({
  //       stakeInfoAccount: stakeInfo,
  //       stakeAccount: stakeAccount,
  //       userTokenAccount: userTokenAccount.address,
  //       mint: mintKeyPair.publicKey,
  //       signer: payer.publicKey
  //     })
  //     .rpc();

  //   console.log("Your transaction signature", tx);

  // });

  // it("DeStake!", async () => {

  //   let userTokenAccount = await getOrCreateAssociatedTokenAccount(
  //     connection, 
  //     payer.payer,
  //     mintKeyPair.publicKey,
  //     payer.publicKey
  //   );

  //   let [stakeInfo] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("stake_info"), payer.publicKey.toBuffer()],
  //     program.programId
  //   );

  //   let [stakeAccount] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("token"), payer.publicKey.toBuffer()],
  //     program.programId
  //   );

  //   let [vaultAccount] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("vault")],
  //     program.programId
  //   )

  //   await mintTo(
  //     connection,
  //     payer.payer,
  //     mintKeyPair.publicKey,
  //     vaultAccount,
  //     payer.payer,
  //     1e21
  //   );

  //   const tx = await program.methods
  //     .destake()
  //     .signers([payer.payer])
  //     .accounts({
  //       stakeAccount: stakeAccount,
  //       stakeInfoAccount: stakeInfo,
  //       userTokenAccount: userTokenAccount.address,
  //       tokenVaultAccount: vaultAccount,
  //       signer: payer.publicKey,
  //       mint: mintKeyPair.publicKey
  //     })
  //     .rpc();

  //   console.log("Your transaction signature", tx);

  // });

  it("createPool!", async () => {
    // Add your test here.

    let [poolInfo] = PublicKey.findProgramAddressSync(
      [Buffer.from("1"), Buffer.from("pool_info")],
      program.programId
    )

    const tx = await program.methods
      .createPools("1")
      .signers([payer.payer])
      .accounts({
        signer: payer.publicKey,
        poolInfoAccount: poolInfo
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });

});
