import type { Meta, StoryObj } from "@storybook/react";
import { Badge } from "./Badge";

const meta: Meta<typeof Badge> = {
  title: "Components/Badge",
  component: Badge,
  tags: ["autodocs"],
  argTypes: {
    status: {
      control: "select",
      options: ["pending", "approved", "rejected", "completed", "cancelled"],
    },
    label: { control: "text" },
  },
};

export default meta;
type Story = StoryObj<typeof Badge>;

export const Pending: Story = { args: { status: "pending" } };
export const Approved: Story = { args: { status: "approved" } };
export const Rejected: Story = { args: { status: "rejected" } };
export const Completed: Story = { args: { status: "completed" } };
export const Cancelled: Story = { args: { status: "cancelled" } };

export const CustomLabel: Story = {
  args: { status: "approved", label: "Refund Approved" },
};

export const AllStatuses: Story = {
  render: () => (
    <div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
      {(["pending", "approved", "rejected", "completed", "cancelled"] as const).map(
        (s) => <Badge key={s} status={s} />
      )}
    </div>
  ),
};
