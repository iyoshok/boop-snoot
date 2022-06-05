

export default function Spacer(props) {
    const rect = props.element.getBoundingClientRect();

    return (
        <>
            <div style={{
                height: `${rect.height}px`,
                width: `${rect.width}px`
            }}></div>
        </>
    )
}