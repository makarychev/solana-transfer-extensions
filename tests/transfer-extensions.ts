import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
  Keypair,
  PublicKey,
  LAMPORTS_PER_SOL,
  ComputeBudgetProgram,
} from "@solana/web3.js";
import {
  ExtensionType,
  TOKEN_2022_PROGRAM_ID,
  getMintLen,
  createInitializeMintInstruction,
  createInitializeTransferHookInstruction,
  createAssociatedTokenAccount,
  createMintToInstruction,
  createTransferCheckedWithTransferHookInstruction,
  getAccount,
  getMint,
  getTransferHook,
  addExtraAccountMetasForExecute,
} from "@solana/spl-token";

import { TransferExtensions } from "../target/types/transfer_extensions";
import { TransferHook } from "../target/types/transfer_hook";
import { assert } from "chai";

describe("transfer-extensions", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = provider.wallet as anchor.Wallet;

  const program = anchor.workspace.TransferExtensions as Program<TransferExtensions>;
  const transferHookProgram = anchor.workspace.TransferHook as Program<TransferHook>;

  it("Initializes Global Program Data", async () => {
    // Add your test here.
    const tx = await program.methods.initializeProgramData().rpc();
    console.log("Your transaction signature", tx);
  });

  // Generate keypair to use as address for the transfer-hook enabled mint
  const mint = new Keypair();
  const decimals = 9;

  it("Create Mint Account with Transfer Hook Extension", async () => {
    const extensions = [ExtensionType.TransferHook];
    const mintLen = getMintLen(extensions);
    const lamports =
      await provider.connection.getMinimumBalanceForRentExemption(mintLen);

    const transaction = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: wallet.publicKey,
        newAccountPubkey: mint.publicKey,
        space: mintLen,
        lamports: lamports,
        programId: TOKEN_2022_PROGRAM_ID,
      }),
      createInitializeTransferHookInstruction(
        mint.publicKey,
        wallet.publicKey,
        transferHookProgram.programId, // Transfer Hook Program ID
        TOKEN_2022_PROGRAM_ID,
      ),
      createInitializeMintInstruction(
        mint.publicKey,
        decimals,
        wallet.payer.publicKey,
        null,
        TOKEN_2022_PROGRAM_ID,
      ),
    );

    const txSig = await sendAndConfirmTransaction(
      provider.connection,
      transaction,
      [wallet.payer, mint],
    );
    console.log(`Transaction Signature: ${txSig}`);
  });

  it("Initializes Mint Counters", async () => {
    const tx = await program.methods.initializeMintCounterIn()
      .accounts({
        mint: mint.publicKey,
      })
      .signers([wallet.payer])
      .rpc();
    console.log("Mint CounterIn transaction signature", tx);

    const mintCounterOutTxSignature = await program.methods.initializeMintCounterOut()
      .accounts({
        mint: mint.publicKey,
      })
      .signers([wallet.payer])
      .rpc();
    console.log("Mint CounterOut transaction signature", mintCounterOutTxSignature);
  });

  const sender = new Keypair();
  let senderTokenAccountPubkey: PublicKey;
  it("Initializes Sender Data", async () => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(sender.publicKey, 1 * LAMPORTS_PER_SOL),
      "confirmed"
    );

    senderTokenAccountPubkey = await createAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      mint.publicKey,
      sender.publicKey,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );

    const tx = await program.methods.initializeWalletCounterIn()
      .accounts({
        mint: mint.publicKey,
        userWallet: sender.publicKey,
      })
      .signers([wallet.payer])
      .rpc();
    console.log("Sender CounterIn transaction signature", tx);

    const senderCounterOutTxSignature = await program.methods.initializeWalletCounterOut()
      .accounts({
        mint: mint.publicKey,
        userWallet: sender.publicKey,
      })
      .signers([wallet.payer])
      .rpc();
    console.log("Sender CounterOut transaction signature", senderCounterOutTxSignature);
  });

  const recipient = new Keypair();
  let recipientTokenAccountPubkey: PublicKey;
  it("Initializes Recipient Data", async () => {
    recipientTokenAccountPubkey = await createAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      mint.publicKey,
      recipient.publicKey,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );

    const tx = await program.methods.initializeWalletCounterIn()
      .accounts({
        mint: mint.publicKey,
        userWallet: recipient.publicKey,
      })
      .signers([wallet.payer])
      .rpc();
    console.log("Recipient CounterIn transaction signature", tx);

    const recipientCounterOutTxSignature = await program.methods.initializeWalletCounterOut()
      .accounts({
        mint: mint.publicKey,
        userWallet: recipient.publicKey,
      })
      .signers([wallet.payer])
      .rpc();
    console.log("Recipient CounterOut transaction signature", recipientCounterOutTxSignature);
  });

  it("Mint Tokens", async () => {
    // 100 tokens
    const amount = 100 * 10 ** decimals;

    const transaction = new Transaction().add(
      createMintToInstruction(
        mint.publicKey,
        senderTokenAccountPubkey,
        wallet.publicKey,
        amount,
        [],
        TOKEN_2022_PROGRAM_ID
      )
    );

    const txSig = await sendAndConfirmTransaction(
      provider.connection,
      transaction,
      [wallet.payer],
      { skipPreflight: true }
    );

    console.log(`Mint Transaction Signature: ${txSig}`);

    const tokenAccount = await getAccount(provider.connection, senderTokenAccountPubkey, undefined, TOKEN_2022_PROGRAM_ID);
    assert.equal(Number(tokenAccount.amount), amount);
  });

  // Account to store extra accounts required by the transfer hook instruction
  it("Create ExtraAccountMetaList Account", async () => {
    const initializeExtraAccountMetaListInstruction = await transferHookProgram.methods
      .initializeExtraAccountMetaList()
      .accounts({
        payer: wallet.publicKey,
        mint: mint.publicKey,
      })
      .instruction();

    const transaction = new Transaction().add(
      initializeExtraAccountMetaListInstruction
    );

    const txSig = await sendAndConfirmTransaction(
      provider.connection,
      transaction,
      [wallet.payer],
      { skipPreflight: true, commitment: "confirmed" }
    );
    console.log("Transaction Signature:", txSig);
  });

  it("Transfers Tokens", async () => {
    // 1 tokens
    const amount = 1 * 10 ** decimals;
    const bigIntAmount = BigInt(amount);

    // Standard token transfer instruction
    const transferInstruction = await createTransferCheckedWithTransferHookInstruction(
      provider.connection,
      senderTokenAccountPubkey,
      mint.publicKey,
      recipientTokenAccountPubkey,
      sender.publicKey,
      bigIntAmount,
      decimals,
      [],
      "confirmed",
      TOKEN_2022_PROGRAM_ID
    );

    const transaction = new Transaction().add(transferInstruction);
    const txSig = await sendAndConfirmTransaction(
      provider.connection,
      transaction,
      [sender],
      { skipPreflight: true }
    );
    console.log(`Transfer Transaction Signature: ${txSig}`);

    const tokenAccount = await getAccount(provider.connection, recipientTokenAccountPubkey, undefined, TOKEN_2022_PROGRAM_ID);
    assert.equal(Number(tokenAccount.amount), amount);
  });

  const recipient2 = new Keypair();
  let recipient2TokenAccountPubkey: PublicKey;
  it("Initializes Recipient2 Data", async () => {
    recipient2TokenAccountPubkey = await createAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      mint.publicKey,
      recipient2.publicKey,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    );

    const tx = await program.methods.initializeWalletCounterIn()
      .accounts({
        mint: mint.publicKey,
        userWallet: recipient2.publicKey,
      })
      .signers([wallet.payer])
      .rpc();
    console.log("Recipient2 CounterIn transaction signature", tx);

    const recipient2CounterOutTxSignature = await program.methods.initializeWalletCounterOut()
      .accounts({
        mint: mint.publicKey,
        userWallet: recipient2.publicKey,
      })
      .signers([wallet.payer])
      .rpc();
    console.log("Recipient2 CounterOut transaction signature", recipient2CounterOutTxSignature);
  });

  it("multiple transfers", async () => {
    // 1 tokens
    const amount1 = 1 * 10 ** decimals;
    const amount2 = 2 * 10 ** decimals;

    const multiTransfersInstruction = program.instruction.multiTransfers(
      new anchor.BN(amount1),
      new anchor.BN(amount2),
      {
        accounts: {
          sourceAccount: senderTokenAccountPubkey,
          destinationAccount1: recipientTokenAccountPubkey,
          destinationAccount2: recipient2TokenAccountPubkey,
          mint: mint.publicKey,
          signer: sender.publicKey,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        },
        signers: [sender],
      })
    const mintInfo = await getMint(
      provider.connection,
      mint.publicKey,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    const transferHook = getTransferHook(mintInfo);
    assert.ok(transferHook);

    await addExtraAccountMetasForExecute(
      provider.connection,
      multiTransfersInstruction,
      transferHook.programId,
      senderTokenAccountPubkey,
      mint.publicKey,
      recipientTokenAccountPubkey,
      sender.publicKey,
      amount1,
      undefined
    );

    await addExtraAccountMetasForExecute(
      provider.connection,
      multiTransfersInstruction,
      transferHook.programId,
      senderTokenAccountPubkey,
      mint.publicKey,
      recipient2TokenAccountPubkey,
      sender.publicKey,
      amount2,
      undefined
    );
    const modifyComputeUnitsInstruction =
      ComputeBudgetProgram.setComputeUnitLimit({
        units: 400000,
      });

    const transaction = new Transaction().add(...[modifyComputeUnitsInstruction, multiTransfersInstruction]);
    try {
      console.log("Going to send transaction");
      const txSig = await sendAndConfirmTransaction(
        provider.connection,
        transaction,
        [sender]
      );
      console.log(`Multi Transfer Transaction Signature: ${txSig}`);
    } catch (error) {
      console.log(error);
      throw error;
    }

    const tokenAccount = await getAccount(provider.connection, recipient2TokenAccountPubkey, undefined, TOKEN_2022_PROGRAM_ID);
    assert.equal(Number(tokenAccount.amount), amount2);
  });
});
