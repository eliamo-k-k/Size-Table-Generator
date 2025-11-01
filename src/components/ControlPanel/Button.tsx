import React, { useState } from "react";
import styled from "styled-components";
import Base from "../../styles/styleAtoms/Button";

interface Props {
  children?: React.ReactElement | string;
  onClick?: (event: React.MouseEvent<HTMLButtonElement>, name?: string) => void;
  type?: "circle" | "pill";
  selected?: boolean;
  name?: string;
  animated?: boolean;
  fontSize?: string;
  disabled?: boolean;
}

export const Button = ({
  children,
  onClick,
  type,
  selected,
  name,
  animated = true,
  fontSize,
  disabled,
}: Props) => {
  const [clicked, setClicked] = useState(false);
  const handleOnClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    if (onClick) {
      onClick(event, name);
    }
    setClicked(true);
    if (animated) setTimeout(() => setClicked(false), 500);
  };
  return (
    <Wrapper
      clicked={animated && clicked}
      buttontype={type}
      onClick={handleOnClick}
      selected={selected}
      fontSize={fontSize}
      disabled={disabled}
    >
      {children}
    </Wrapper>
  );
};

interface StyleProps {
  fontSize?: string;
}

const Wrapper = styled(Base)<StyleProps>`
  font-size: ${(props) => (props.fontSize ? `${props.fontSize}` : "1rem")};
`;
