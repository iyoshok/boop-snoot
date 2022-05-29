import './log_element.css';

export default function LogElement(props) {
    const paddedString = (val: number) => {
        return val > 9 ?
            val.toString() :
            `0${val}`;
    };

    const formatDate = (time: Date) => {
        const days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        return `${days[time.getDay()]} ${paddedString(time.getDate())}.${paddedString(time.getMonth() + 1)}. // ${paddedString(time.getHours())}:${paddedString(time.getMinutes())}:${paddedString(time.getSeconds())}`
    }

    return (
        <li class='log-entry'>
            <div class="log-time"><b>{formatDate(props.time)}</b></div>
            <div class="log-content">{props.content}</div>
            <hr />
        </li>
    )
}