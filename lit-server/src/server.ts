import "dotenv/config";
import express, { Request, Response } from "express";
import * as ethers from "ethers";
import { LitNodeClient } from "@lit-protocol/lit-node-client";
import { LIT_RPC, LIT_ABILITY } from "@lit-protocol/constants";
import { LitActionResource, LitPKPResource } from "@lit-protocol/auth-helpers";
import { ExecuteJsResponse } from "@lit-protocol/types";

const LIT_NETWORK = "datil-test" as const;
const PKP_PUBLIC_KEY = process.env.PKP_PUBLIC_KEY;
const PKP_WALLET_OWNER_PRIVATE_KEY = process.env.PKP_WALLET_OWNER_PRIVATE_KEY;
const PORT = process.env.PORT || 3050;

interface ExecuteLitActionRequest {
    ipfsCid: string;
    jsParams: Record<string, any>;
}

type ExecuteLitActionResponse = ExecuteJsResponse;

const app = express();
app.use(express.json());

let litNodeClient: LitNodeClient | null = null;
let sessionSigs: any = null;

async function initializeLitClient() {
    console.log("Initializing Lit Protocol client...");

    litNodeClient = new LitNodeClient({
        litNetwork: LIT_NETWORK,
        debug: false,
    });

    await litNodeClient.connect();
    console.log("✓ Connected to Lit Network");

    if (!PKP_WALLET_OWNER_PRIVATE_KEY) {
        throw new Error("PKP_WALLET_OWNER_PRIVATE_KEY not set in environment");
    }

    const wallet = new ethers.Wallet(
        PKP_WALLET_OWNER_PRIVATE_KEY,
        new ethers.providers.JsonRpcProvider(LIT_RPC.CHRONICLE_YELLOWSTONE)
    );

    console.log("Generating session signatures...");

    sessionSigs = await litNodeClient.getSessionSigs({
        pkpPublicKey: PKP_PUBLIC_KEY,
        chain: "ethereum",
        resourceAbilityRequests: [
            {
                resource: new LitActionResource("*"),
                ability: LIT_ABILITY.LitActionExecution,
            },
            {
                resource: new LitPKPResource("*"),
                ability: LIT_ABILITY.PKPSigning,
            },
        ],
        authNeededCallback: async ({
            uri,
            expiration,
            resourceAbilityRequests,
        }) => {
            const { createSiweMessage, generateAuthSig } = await import(
                "@lit-protocol/auth-helpers"
            );

            const toSign = await createSiweMessage({
                uri,
                expiration,
                litNodeClient: litNodeClient!,
                resources: resourceAbilityRequests,
                walletAddress: wallet.address,
                nonce: await litNodeClient!.getLatestBlockhash(),
            });

            return generateAuthSig({
                signer: wallet,
                toSign,
            });
        },
    });

    console.log("✓ Session signatures generated");
    console.log(`✓ Lit Actions server ready on port ${PORT}`);
}

app.post(
    "/execute",
    (
        req: Request<{}, {}, ExecuteLitActionRequest>,
        res: Response<ExecuteLitActionResponse>
    ) => {
        if (!litNodeClient || !sessionSigs) {
            return res.status(503).end();
        }

        const { ipfsCid, jsParams } = req.body;

        if (!ipfsCid) {
            return res.status(400).end();
        }

        if (!jsParams || typeof jsParams !== "object") {
            return res.status(400).end();
        }

        console.log(`Executing Lit Action (IPFS: ${ipfsCid})`);

        litNodeClient
            .executeJs({
                ipfsId: ipfsCid,
                sessionSigs,
                jsParams: {
                    ...jsParams,
                    publicKey: PKP_PUBLIC_KEY,
                },
            })
            .then((result) => {
                console.log("✓ Lit Action executed successfully");

                return res.json(result);
            })
            .catch((error) => {
                console.error("Error executing Lit Action:", error);

                return res.status(500).send(error.message || "Unknown error");
            });
    }
);

app.get("/health", (_req: Request, res: Response) => {
    res.json({
        status: "ok",
        litConnected: litNodeClient?.ready || false,
        hasSessionSigs: !!sessionSigs,
    });
});

initializeLitClient()
    .then(() => {
        app.listen(PORT, () => {
            console.log(
                `\n🚀 Lit Actions HTTP server listening on http://localhost:${PORT}`
            );
            console.log(`   POST /execute - Execute Lit Action`);
            console.log(`   GET  /health  - Health check\n`);
        });
    })
    .catch((error) => {
        console.error("Failed to initialize:", error);
        process.exit(1);
    });
