import React, { useState } from "react";
import RowFlex from "../../styles/styleAtoms/RowFlexWrapper";
import ColumnFlex from "../../styles/styleAtoms/ColumnFlexWrapper";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useAtom } from "jotai";
import {
  itemMetasAtom,
  showLoadingLogoAtom,
  statusInfoAtom,
} from "../../lib/store";
import { Button } from "./Button";
import styled from "styled-components";
import { Color } from "../../styles/Color";
import { trimHomePath } from "../../lib/utility";
import {
  CommandInvokeError,
  ProcessResponse,
  ProcessStatePayload,
} from "../../types";

interface OpenExcelFileProps {
  onExcelLoaded: (data: any[][]) => void;
}

export const OpenExcelFile: React.FC<OpenExcelFileProps> = ({
  onExcelLoaded,
}) => {
  const [, setItemMetas] = useAtom(itemMetasAtom);
  const [, setShowLoadingLogo] = useAtom(showLoadingLogoAtom);
  const [, setStatusInfo] = useAtom(statusInfoAtom);
  const [excelPath, setExcelPath] = useState<string>("");
  const [filePath, setFilePath] = useState("");
  const [fileName, setFileName] = useState<string>("");

  const handleOpenFileOnClick = async () => {
    try {
      const path = await open();
      if (!path) return;

      const pathStr = path as string;
      setExcelPath(pathStr);

      const trimmedPath = await trimHomePath(pathStr);
      setFilePath(trimmedPath);

      const name = pathStr.split("/").pop() || "";
      setFileName(name);

      // 一度プレビューと結果をクリア
      onExcelLoaded([]);
      setItemMetas([]);
      setStatusInfo({ type: "normal", content: "done" });
    } catch (e) {
      setStatusInfo({ type: "error", content: "プレビュー取得失敗" });
    }
  };

  const handleProcessClick = async () => {
    if (!excelPath) {
      setStatusInfo({
        type: "error",
        content: "先にファイルを選択してください",
      });
      return;
    }
    setShowLoadingLogo(true);
    setStatusInfo({ type: "normal", content: "文件处理中" });

    let unlisten: (() => void) | null = null;
    try {
      // Rust 側の update-state イベントを listen して進行状況を UI に反映
      unlisten = await listen<ProcessStatePayload>("update-state", (event) => {
        setStatusInfo({
          type: "normal",
          content: event.payload.state,
        });
      });

      const res = (await invoke("process_excel_file", {
        excelPath: excelPath,
      })) as ProcessResponse;

      setItemMetas(res.item_meta);
      onExcelLoaded([]); // プレビューをクリア
      setStatusInfo({ type: "normal", content: "done" });
    } catch (e) {
      const message =
        (e as any)?.toString?.() ??
        ((e as any)?.message as CommandInvokeError) ??
        "process失敗";
      setStatusInfo({ type: "error", content: message as CommandInvokeError });
    } finally {
      if (unlisten) {
        unlisten();
      }
      setShowLoadingLogo(false);
    }
  };

  return (
    <Wrapper>
      <RowWrapper>
        <Button onClick={handleOpenFileOnClick}>打开源文件</Button>
      </RowWrapper>
      <RowWrapper>
        <Text>现正打开：{filePath}</Text>
      </RowWrapper>
      <RowWrapper>
        <Button onClick={handleProcessClick} disabled={!fileName}>
          生成開始
        </Button>
      </RowWrapper>
    </Wrapper>
  );
};

const Wrapper = styled(ColumnFlex)`
  width: 80%;
  max-width: 300px;
  border: 0px solid;
  border-radius: 30px;
  background-color: ${Color.SUB};
  min-height: 100px;
`;

const RowWrapper = styled(RowFlex)`
  width: 100%;
  gap: 15px;
`;
const Text = styled.div`
  overflow-wrap: break-word;
  min-width: 70%;
  max-width: 90%;
`;
