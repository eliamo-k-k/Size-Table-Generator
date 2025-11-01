import React, { useState } from "react";
import RowFlex from "../../styles/styleAtoms/RowFlexWrapper";
import ColumnFlex from "../../styles/styleAtoms/ColumnFlexWrapper";
import { open } from "@tauri-apps/api/dialog";
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
import { Preview } from "../Preview";

interface OpenExcelFileProps {
  onExcelLoaded: (data: any[][]) => void;
}

export const OpenExcelFile: React.FC<OpenExcelFileProps> = ({
  onExcelLoaded,
}) => {
  const [itemMetas, setItemMetas] = useAtom(itemMetasAtom);
  const [, setShowLoadingLogo] = useAtom(showLoadingLogoAtom);
  const [, setStatusInfo] = useAtom(statusInfoAtom);
  const [filePath, setFilePath] = useState("");
  const [fileName, setFileName] = useState<string>("");

  const handleOpenFileOnClick = async () => {
    const path = await open();
    if (!path) return;
    const trimmedPath = await trimHomePath(path as string);
    setFilePath(trimmedPath);
    const name = (path as string).split("/").pop() || "";
    setFileName(name);
    setShowLoadingLogo(true);
    setStatusInfo({ type: "normal", content: "アップロード中" });
    try {
      // 本来はFile APIでファイルを取得する必要があるが、ここはダミーで送信
      const file = new File([new Blob(["dummy"])], name);
      const formData = new FormData();
      formData.append("file", file);
      const uploadRes = await fetch("http://localhost:8787/upload", {
        method: "POST",
        body: formData,
      });
      if (!uploadRes.ok) {
        setStatusInfo({ type: "error", content: "アップロード失敗" });
        setShowLoadingLogo(false);
        return;
      }
      // /upload成功時はローカルパース済みデータでプレビュー
      onExcelLoaded([]); // ここは親でローカルパースしたデータを渡す想定
      setItemMetas([]); // itemMetasAtomをクリア
    } catch (e) {
      setStatusInfo({ type: "error", content: "プレビュー取得失敗" });
    } finally {
      setShowLoadingLogo(false);
    }
  };

  const handleProcessClick = async () => {
    if (!fileName) {
      setStatusInfo({
        type: "error",
        content: "先にファイルを選択してください",
      });
      return;
    }
    setShowLoadingLogo(true);
    setStatusInfo({ type: "normal", content: "文件处理中" });
    try {
      const res = await fetch("http://localhost:8787/process", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ filename: fileName }),
      });
      if (!res.ok) {
        setStatusInfo({ type: "error", content: await res.text() });
        setShowLoadingLogo(false);
        return;
      }
      const data = await res.json();
      setItemMetas(data.item_meta);
      onExcelLoaded([]); // プレビューをクリア
      setStatusInfo({ type: "normal", content: "done" });
    } catch (e) {
      setStatusInfo({ type: "error", content: "process失敗" });
    } finally {
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
      {itemMetas && itemMetas.length > 0 && (
        <div style={{ width: "100%", marginTop: 16 }}>
          <Preview />
        </div>
      )}
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
