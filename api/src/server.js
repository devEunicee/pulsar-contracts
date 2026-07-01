import express from "express";
import { errorMiddleware } from "./middleware/errors.js";
import merchantsRouter from "./routes/merchants.js";
import paymentsRouter from "./routes/payments.js";
import tfaRouter from "./routes/tfa.js";

const app = express();

app.use(express.json());

app.get("/health", (_req, res) => res.json({ status: "ok" }));
app.use("/api/merchants", merchantsRouter);
app.use("/api/payments", paymentsRouter);
app.use("/api/auth/2fa", tfaRouter);

app.use(errorMiddleware);

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => console.log(`Pulsar API running on port ${PORT}`));
