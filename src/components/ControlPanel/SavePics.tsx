import React from "react";
import styled from "styled-components";
import { Color } from "../../styles/Color";
import { Button } from "./Button";
import RowFlex from "../../styles/styleAtoms/RowFlexWrapper";
import ColumnFlex from "../../styles/styleAtoms/ColumnFlexWrapper";
import { saveDirAtom, statusInfoAtom, tableIdsAtom } from "../../lib/store";
import { useAtom } from "jotai";
import { checkDirThenCreate, saveElementToPath } from "../../lib/saveFile";
import { open as shellOpen } from "@tauri-apps/plugin-shell";
import { triggerTempStatusPanel } from "../../lib/utility";

export const SavePics = () => {
  const [saveDir] = useAtom(saveDirAtom);
  const [, setStatusInfo] = useAtom(statusInfoAtom);
  const [tableIds] = useAtom(tableIdsAtom);
  const handleSaveTableButtonOnClick = async () => {
    if (tableIds.length === 0) {
      triggerTempStatusPanel(setStatusInfo, "error", "请先打开源文件");
      return;
    }
    if (saveDir.length === 0) {
      triggerTempStatusPanel(setStatusInfo, "error", "请设置保存文件夹");
      return;
    }
    for (let i = 0; i < tableIds.length; i++) {
      const id = tableIds[i];
      const baseDir = `${saveDir}/${id}`;
      const fileDir = `${baseDir}/size_${id}.jpg`;
      await checkDirThenCreate(baseDir);
      setStatusInfo({ type: "normal", content: `正在保存 size_${id}.jpg` });
      await saveElementToPath(id, fileDir);
    }
    setStatusInfo({ type: "normal", content: "done" });
    shellOpen(saveDir);
  };
  return (
    <Wrapper>
      <RowWrapper>
        <Button onClick={handleSaveTableButtonOnClick}>保存尺码表</Button>
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
`;
