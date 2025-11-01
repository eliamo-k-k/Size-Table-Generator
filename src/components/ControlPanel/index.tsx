import React, { useState } from "react";
import styled from "styled-components";
import WrapperBase from "../../styles/styleAtoms/ColumnFlexWrapper";
import { Color } from "../../styles/Color";
import Logo from "../../LOGOTEXTJPN256.png";
import { OpenExcelFile } from "./OpenExcelFile";
import { SelectSaveDir } from "./SelectSaveDir";
import { SavePics } from "./SavePics";
import { useAtom } from "jotai";
import { appVersionAtom } from "../../lib/store";
import { ExcelPreview } from "./ExcelPreview";

export const ControlPanel = () => {
  const [appVersion] = useAtom(appVersionAtom);
  const [excelData, setExcelData] = useState<any[][]>([]);
  return (
    <Layout>
      <Wrapper>
        <LogoWrapper src={Logo} alt="logo" />
      </Wrapper>
      <Wrapper>
        <OpenExcelFile onExcelLoaded={setExcelData} />
        <SelectSaveDir />
        <SavePics />
        <ExcelPreview data={excelData} />
      </Wrapper>
      <AppVersionWrapper>v{appVersion}</AppVersionWrapper>
    </Layout>
  );
};

const Wrapper = styled(WrapperBase)`
  height: 100%;
  width: 100%;
  gap: 80px;
  border-left: 1px solid ${Color.EliamoGold};
`;

const AppVersionWrapper = styled.div`
  position: absolute;
  bottom: 0;
  right: 0;
  color: gray;
  font-size: 0.5rem;
  transform: translateX(-5px) translateY(-5px);
`;

const Layout = styled.div`
  position: relative;
  display: grid;
  height: 100%;
  width: 100%;
  grid-template-rows: 1fr 6fr;
`;

const LogoWrapper = styled.img`
  width: 70%;
`;
