import { useAtom } from "jotai";
import React from "react";
import styled from "styled-components";
import { itemMetasAtom } from "../../lib/store";
import Base from "../../styles/styleAtoms/ColumnFlexWrapper";
import { ItemMeta } from "../../types";
import { ListItem } from "./ListItem";

export const ListView = () => {
  const [itemMetas] = useAtom(itemMetasAtom);
  return (
    <Wrapper>
      {itemMetas.map((itemMeta: ItemMeta) => (
        <ListItem key={`${itemMeta.code}`} itemMeta={itemMeta} />
      ))}
    </Wrapper>
  );
};

const Wrapper = styled(Base)`
  justify-content: flex-start;
  /* 親のプレビューエリアの高さに合わせる */
  height: 100%;
  overflow-y: auto;
  scrollbar-width: none;
  &::-webkit-scrollbar {
    display: none;
  }
  & > :first-child {
    margin-top: 30px;
  }

  & > :last-child {
    margin-bottom: 30px;
  }
`;
