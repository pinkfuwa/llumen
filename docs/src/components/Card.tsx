import React, { ReactNode } from 'react';
import Link from '@docusaurus/Link';
import styles from './Card.module.css';

interface CardProps {
  title?: string;
  icon?: string;
  href?: string;
  children?: ReactNode;
}

export function Card({ title, icon, href, children }: CardProps) {
  const content = (
    <div className={styles.card}>
      {icon && <div className={styles.icon}>{icon}</div>}
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
    <div className={styles.cardGroup} style={{ gridTemplateColumns: `repeat(${cols}, 1fr)` }}>
      {children}
    </div>
  );
}
