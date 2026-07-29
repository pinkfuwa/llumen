import React from "react";
import MDXComponents from "@theme-original/MDXComponents";
import { Card, CardGroup } from "@site/src/components/Card";
import { Slide } from "@site/src/components/Slide";
import { Scroll } from "@site/src/components/Scroll";
import Admonition from "@theme/Admonition";

// Mintlify compatibility components
const Note = ({ children }: { children: React.ReactNode }) => (
  <Admonition type="note" children={children} />
);
const Warning = ({ children }: { children: React.ReactNode }) => (
  <Admonition type="warning" children={children} />
);
const Info = ({ children }: { children: React.ReactNode }) => (
  <Admonition type="info" children={children} />
);
const Tip = ({ children }: { children: React.ReactNode }) => (
  <Admonition type="tip" children={children} />
);

// Tabs support - wrapper to convert Mintlify format to Docusaurus
import DocTabs from "@theme/Tabs";
import DocTabItem from "@theme/TabItem";

const Tab = ({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) => (
  <DocTabItem value={title} label={title} children={children} />
);

const Tabs = ({ children }: { children: React.ReactNode }) => {
  return <DocTabs children={children} />;
};

// Simple Accordion implementation
const Accordion = ({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) => (
  <details style={{ marginBottom: "1rem" }}>
    <summary
      style={{ cursor: "pointer", fontWeight: "bold", padding: "0.5rem 0" }}
    >
      {title}
    </summary>
    <div style={{ padding: "0.5rem 0 0 1rem" }}>{children}</div>
  </details>
);

const AccordionGroup = ({ children }: { children: React.ReactNode }) => (
  <div>{children}</div>
);

export default {
  ...MDXComponents,
  Card,
  CardGroup,
  Slide,
  Scroll,
  Note,
  Warning,
  Info,
  Tip,
  Tabs,
  Tab,
  Accordion,
  AccordionGroup,
};
