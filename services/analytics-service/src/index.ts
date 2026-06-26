import express from "express";
import { createAnalyticsRouter } from "./api";

const PORT = Number(process.env.PORT ?? 3100);
const DATABASE_URL = process.env.DATABASE_URL ?? "";

if (!DATABASE_URL) {
  console.error("DATABASE_URL environment variable is required");
  process.exit(1);
}

const app = express();
app.use(express.json());
app.use(createAnalyticsRouter(DATABASE_URL));

app.listen(PORT, () => {
  console.log(`[analytics] Service listening on port ${PORT}`);
});
