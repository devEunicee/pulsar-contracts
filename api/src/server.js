import express from "express";
import { errorMiddleware } from "./middleware/errors.js";
import { corsMiddleware } from "./middleware/cors.js";
import merchantsRouter from "./routes/merchants.js";
import paymentsRouter from "./routes/payments.js";
import apiKeysRouter from "./routes/apiKeys.js";
import oauthRouter from "./routes/oauth.js";

const app = express();

// Apply CORS before any route handlers so preflight requests are handled first.
app.use(corsMiddleware);

app.use(express.json());

app.get("/health", (_req, res) => res.json({ status: "ok" }));
app.use("/api/merchants", merchantsRouter);
app.use("/api/payments", paymentsRouter);
app.use("/api/keys", apiKeysRouter);
app.use("/oauth", oauthRouter);

app.use(errorMiddleware);

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => console.log(`Pulsar API running on port ${PORT}`));
