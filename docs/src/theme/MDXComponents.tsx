import React from "react";
import MDXComponents from "@theme-original/MDXComponents";
import { Card, CardGroup } from "@site/src/components/Card";
import Admonition from "@theme/Admonition";

// Mintlify compatibility components
const Note = ({ children }) => <Admonition type="note">{children}</Admonition>;
const Warning = ({ children }) => (
  <Admonition type="warning">{children}</Admonition>
);
const Info = ({ children }) => <Admonition type="info">{children}</Admonition>;
const Tip = ({ children }) => <Admonition type="tip">{children}</Admonition>;

// Tabs support - wrapper to convert Mintlify format to Docusaurus
import DocTabs from "@theme/Tabs";
import DocTabItem from "@theme/TabItem";

const Tab = ({ title, children }) => (
  <DocTabItem value={title} label={title}>
    {children}
  </DocTabItem>
);

const Tabs = ({ children }) => {
  return <DocTabs>{children}</DocTabs>;
};

// Simple Accordion implementation
const Accordion = ({ title, children }) => (
  <details style={{ marginBottom: "1rem" }}>
    <summary
      style={{ cursor: "pointer", fontWeight: "bold", padding: "0.5rem 0" }}
    >
      {title}
    </summary>
    <div style={{ padding: "0.5rem 0 0 1rem" }}>{children}</div>
  </details>
);

const AccordionGroup = ({ children }) => <div>{children}</div>;

export default {
  ...MDXComponents,
  Card,
  CardGroup,
  Note,
  Warning,
  Info,
  Tip,
  Tabs,
  Tab,
  Accordion,
  AccordionGroup,
};
