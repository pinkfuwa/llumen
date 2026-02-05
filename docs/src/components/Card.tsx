import React, { ReactNode } from "react";
import Link from "@docusaurus/Link";
import * as LucideIcons from "lucide-react";
import styles from "./Card.module.css";

interface CardProps {
  title?: string;
  icon?: string;
  href?: string;
  children?: ReactNode;
}

// Map of common icon names to Lucide icons
const iconMap: Record<string, keyof typeof LucideIcons> = {
  "shield-check": "ShieldCheck",
  "feather": "Feather",
  "bolt": "Zap",
  "messages": "MessagesSquare",
  "message": "MessageSquare",
  "image": "Image",
  "palette": "Palette",
  "gauge-high": "Gauge",
  "download": "Download",
  "gear": "Settings",
  "rocket": "Rocket",
  "plug": "Plug",
  "docker": "Container",
  "wrench": "Wrench",
  "bug": "Bug",
  "lightbulb": "Lightbulb",
  "book": "Book",
  "code": "Code",
  "magnifying-glass": "Search",
  "microscope": "Microscope",
  "upload": "Upload",
  "server": "Server",
  "network-wired": "Network",
  "cloud": "Cloud",
  "sitemap": "Network",
};

export function Card({ title, icon, href, children }: CardProps) {
  const IconComponent = icon && iconMap[icon] 
    ? LucideIcons[iconMap[icon] as keyof typeof LucideIcons] as React.ComponentType<{ size?: number; className?: string }>
    : null;

  const content = (
    <div className={styles.card}>
      {IconComponent && (
        <div className={styles.icon}>
          <IconComponent size={24} />
        </div>
      )}
      {title && <h3 className={styles.title}>{title}</h3>}
      {children && <div className={styles.content}>{children}</div>}
    </div>
  );

  if (href) {
    return (
      <Link to={href} className={styles.cardLink}>
        {content}
      </Link>
    );
  }

  return content;
}

interface CardGroupProps {
  cols?: number;
  children: ReactNode;
}

export function CardGroup({ cols = 2, children }: CardGroupProps) {
  return (
    <div
      className={styles.cardGroup}
      style={{ gridTemplateColumns: `repeat(${cols}, 1fr)` }}
    >
      {children}
    </div>
  );
}
