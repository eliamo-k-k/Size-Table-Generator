import React from "react";
import {
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
} from "@mui/material";

interface ExcelPreviewProps {
  data: any[][];
}

export const ExcelPreview: React.FC<ExcelPreviewProps> = ({ data }) => {
  if (!data || data.length === 0) return null;

  return (
    <TableContainer component={Paper} sx={{ my: 2 }}>
      <Table size="small">
        <TableHead>
          <TableRow>
            {data[0].map((cell, idx) => (
              <TableCell key={idx}>{cell}</TableCell>
            ))}
          </TableRow>
        </TableHead>
        <TableBody>
          {data.slice(1).map((row, i) => (
            <TableRow key={i}>
              {row.map((cell, j) => (
                <TableCell key={j}>{cell}</TableCell>
              ))}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );
};
