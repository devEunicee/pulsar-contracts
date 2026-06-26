import type { Meta, StoryObj } from "@storybook/react";
import { fn } from "@storybook/test";
import { PaymentCard } from "./PaymentCard";

const meta: Meta<typeof PaymentCard> = {
  title: "Components/PaymentCard",
  component: PaymentCard,
  tags: ["autodocs"],
  args: { onViewDetails: fn() },
};

export default meta;
type Story = StoryObj<typeof PaymentCard>;

const base = {
  orderId: "ORDER-2024-001",
  merchantName: "Pulsar Store",
  amount: "1,000 XLM",
  date: "2024-06-01T10:30:00Z",
};

export const Completed: Story = { args: { ...base, status: "completed" } };
export const Pending: Story = { args: { ...base, status: "pending" } };
export const Rejected: Story = { args: { ...base, status: "rejected" } };
export const NoAction: Story = {
  args: { ...base, status: "completed", onViewDetails: undefined },
};
