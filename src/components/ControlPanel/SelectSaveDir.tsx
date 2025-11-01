import React, { useState } from "react";
import styled from "styled-components";
import { Color } from "../../styles/Color";
import ColumnFlex from "../../styles/styleAtoms/ColumnFlexWrapper";
import RowFlex from "../../styles/styleAtoms/RowFlexWrapper";
import { Button } from "./Button";
import { open } from "@tauri-apps/plugin-dialog";
import { saveDirAtom } from "../../lib/store";
import { useAtom } from "jotai";
import { trimHomePath } from "../../lib/utility";

export const SelectSaveDir = () => {
  const [, setSaveDir] = useAtom(saveDirAtom);
  const [showingDir, setShowingDir] = useState("");
  const handleSavePathButtonOnClick = async () => {
    const path = await open({ directory: true });
    if (!path) return;
    setSaveDir(path as string);
    const trimmed = await trimHomePath(path as string);
    setShowingDir(trimmed);
  };
  return (
    <Wrapper>
      <RowWrapper>
        <Button onClick={handleSavePathButtonOnClick}>选择保存文件夹</Button>
      </RowWrapper>
      <RowWrapper>
        <Text>保存地址：{showingDir} </Text>
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

const Text = styled.div`
  overflow-wrap: break-word;
  min-width: 70%;
  max-width: 90%;
`;
