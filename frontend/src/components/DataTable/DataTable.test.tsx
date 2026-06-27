import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DataTable, ColumnDef } from "./DataTable";

interface Row {
  id: string;
  name: string;
  amount: number;
}

const columns: ColumnDef<Row>[] = [
  { key: "name", header: "Name", sortable: true, getValue: (r) => r.name },
  { key: "amount", header: "Amount", sortable: true, getValue: (r) => r.amount },
];

const rows: Row[] = [
  { id: "1", name: "Alice", amount: 100 },
  { id: "2", name: "Bob", amount: 200 },
];

function getRowKey(r: Row) {
  return r.id;
}

describe("DataTable", () => {
  it("renders table with caption and column headers", () => {
    render(
      <DataTable
        columns={columns}
        rows={rows}
        caption="Payments"
        getRowKey={getRowKey}
      />
    );
    expect(screen.getByText("Payments")).toBeInTheDocument();
    expect(screen.getByText("Name")).toBeInTheDocument();
    expect(screen.getByText("Amount")).toBeInTheDocument();
  });

  it("renders row data", () => {
    render(
      <DataTable columns={columns} rows={rows} getRowKey={getRowKey} />
    );
    expect(screen.getByText("Alice")).toBeInTheDocument();
    expect(screen.getByText("Bob")).toBeInTheDocument();
  });

  it("shows empty state when no rows", () => {
    render(<DataTable columns={columns} rows={[]} getRowKey={getRowKey} />);
    expect(screen.getByText("No data available.")).toBeInTheDocument();
  });

  it("calls onSortChange when sortable header clicked", () => {
    const onSort = jest.fn();
    render(
      <DataTable
        columns={columns}
        rows={rows}
        getRowKey={getRowKey}
        onSortChange={onSort}
      />
    );
    fireEvent.click(screen.getByText("Name"));
    expect(onSort).toHaveBeenCalledWith("name", "asc");
  });

  it("sets aria-sort attribute on sorted column header", () => {
    render(
      <DataTable
        columns={columns}
        rows={rows}
        getRowKey={getRowKey}
        sortColumn="name"
        sortDirection="asc"
      />
    );
    expect(screen.getByText("Name").closest("th")).toHaveAttribute(
      "aria-sort",
      "ascending"
    );
  });

  it("renders checkboxes when selectable", () => {
    render(
      <DataTable
        columns={columns}
        rows={rows}
        getRowKey={getRowKey}
        selectable
        selectedKeys={new Set()}
      />
    );
    const checkboxes = screen.getAllByRole("checkbox");
    // 1 select-all + 2 row checkboxes
    expect(checkboxes).toHaveLength(3);
  });

  it("calls onSelectionChange when row checkbox is toggled", async () => {
    const onChange = jest.fn();
    render(
      <DataTable
        columns={columns}
        rows={rows}
        getRowKey={getRowKey}
        selectable
        selectedKeys={new Set()}
        onSelectionChange={onChange}
      />
    );
    const [, firstRowCheckbox] = screen.getAllByRole("checkbox");
    await userEvent.click(firstRowCheckbox);
    expect(onChange).toHaveBeenCalledWith(new Set(["1"]));
  });

  it("selects all when select-all checkbox is clicked", async () => {
    const onChange = jest.fn();
    render(
      <DataTable
        columns={columns}
        rows={rows}
        getRowKey={getRowKey}
        selectable
        selectedKeys={new Set()}
        onSelectionChange={onChange}
      />
    );
    const [selectAll] = screen.getAllByRole("checkbox");
    await userEvent.click(selectAll);
    expect(onChange).toHaveBeenCalledWith(new Set(["1", "2"]));
  });

  it("has role=grid on the table element", () => {
    render(
      <DataTable columns={columns} rows={rows} getRowKey={getRowKey} />
    );
    expect(screen.getByRole("grid")).toBeInTheDocument();
  });

  it("triggers sort on Enter keypress on sortable header", () => {
    const onSort = jest.fn();
    render(
      <DataTable
        columns={columns}
        rows={rows}
        getRowKey={getRowKey}
        onSortChange={onSort}
      />
    );
    const nameHeader = screen.getByText("Name").closest("th")!;
    fireEvent.keyDown(nameHeader, { key: "Enter" });
    expect(onSort).toHaveBeenCalledWith("name", "asc");
  });
});
