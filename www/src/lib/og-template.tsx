import type { ReactNode } from "react";

const PULSAR_GREEN = "#9AE600";

function Logo() {
  return (
    <svg
      width="48"
      height="48"
      viewBox="0 0 523 519"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M236.61 187.94L6.73213 515.501H255.357L518.732 515.501C518.732 515.501 396.479 -165.579 319.157 43.3186C278.004 154.497 236.61 187.94 236.61 187.94ZM518.732 515.501L426.781 268.285L357.818 244.8L313.932 143.441L236.61 187.94"
        stroke={PULSAR_GREEN}
        strokeWidth="7"
      />
    </svg>
  );
}

export function OgTemplate({
  title,
  description,
  site,
  icon,
}: {
  title: string;
  description?: string;
  site?: string;
  icon?: ReactNode;
}) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        justifyContent: "center",
        width: "100%",
        height: "100%",
        color: "white",
        padding: "64px",
        gap: "32px",
        backgroundColor: "#0c0c0c",
      }}
    >
      <div
        style={{
          display: "flex",
          flexDirection: "row",
          alignItems: "center",
          gap: "16px",
        }}
      >
        {icon ?? <Logo />}
        {site && <p style={{ fontSize: "40px", fontWeight: 500, margin: 0 }}>{site}</p>}
      </div>

      <div
        style={{
          width: "2px",
          height: "100%",
          backgroundColor: PULSAR_GREEN,
          marginInline: "32px",
        }}
      ></div>

      <div
        style={{
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          gap: "16px",
        }}
      >
        <p
          style={{
            fontWeight: 600,
            fontSize: "52px",
            margin: 0,
            lineHeight: 1.1,
            textAlign: "center",
          }}
        >
          {title}
        </p>

        {description && (
          <>
            <p
              style={{
                fontSize: "30px",
                color: "rgba(240,240,240,0.8)",
                margin: 0,
                lineHeight: 1.3,
                textAlign: "center",
              }}
            >
              {description}
            </p>
          </>
        )}
      </div>
    </div>
  );
}
