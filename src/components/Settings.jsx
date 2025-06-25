import './Settings.css';
import {useEffect, useState} from 'react';

export const Settings = ({startLoop, currentIndex}) => {
    const [urls, setUrls] = useState([""]);
    const [timeBetween, setTimeBetween] = useState(0);

    useEffect(() => {
        const savedSettings = localStorage.getItem('settings');
        if (savedSettings) {
            const {urls, timeBetween} = JSON.parse(savedSettings);
            setUrls(urls.length > 0 ? urls : [""]);
            setTimeBetween(timeBetween || 0);
        }
    }, []);

    const changeHandler = (e, index) => {
        const val = e.target.value;
        if (val === "") {
            console.log(`Removing URL at index ${index}`);
            urls.splice(index, 1);
            if (urls.length === 0) {
                urls.push("");
            }
            setUrls([...urls]);
            return;
        }
        const newUrls = [...urls];
        newUrls[index] = val;
        if (index === newUrls.length - 1 && newUrls[index] !== "") {
            newUrls.push("");
        }
        setUrls(newUrls);
    }

    const timeChangeHandler = (e) => {
        const val = e.target.value;
        if (val === "") {
            setTimeBetween(0);
            return;
        }
        const time = parseFloat(val);
        if (isNaN(time) || time <= 0) {
            return;
        }
        setTimeBetween(time);
    }

    const applyHandler = () => {
        const displayUrls = [];
        urls.forEach(url => {
            if (url === "") return;
            if (!url.startsWith("http://") && !url.startsWith("https://")) {
                url = "https://" + url;
            }
            if (!url.includes(".")) {
                alert("Please enter valid URLs.");
                return;
            }
            displayUrls.push(url);
        });
        if (displayUrls.length === 0) {
            alert("Please enter at least one valid URL.");
            return;
        }
        if (timeBetween <= 0) {
            alert("Please enter a valid time between URLs.");
            return;
        }
        startLoop(displayUrls, timeBetween * 1000);
        localStorage.setItem('settings', JSON.stringify({urls, timeBetween}));
    };

    return (
        <div className="settings-container">
            <div className="settings-card">
                <h2 className="settings-title">Settings</h2>
                <label className="settings-label">URLs to display:</label>
                <ul className="settings-url-list">
                    {urls.map((url, index) => (
                        <li
                            key={index}
                            className={`settings-url-item${index === currentIndex ? ' settings-url-item--active' : ''}`}
                        >
                            <input
                                className="settings-input"
                                type="text"
                                value={url}
                                placeholder={`Enter URL #${index + 1}`}
                                onChange={(e) => changeHandler(e, index)}
                            />
                        </li>
                    ))}
                </ul>
                <label className="settings-label" style={{marginTop: '1em'}}>Time between URLs (seconds):</label>
                <input
                    className="settings-input"
                    type="text"
                    placeholder="Time between URLs (seconds)"
                    value={timeBetween}
                    onChange={timeChangeHandler}
                />
                <button className="settings-apply-btn" onClick={applyHandler}>
                    <span style={{marginRight: 6, display: 'inline-block', verticalAlign: 'middle'}}>
                        <svg width="18" height="18" viewBox="0 0 20 20" fill="none">
                            <rect width="20" height="20" rx="5" fill="#396cd8"/>
                            <path d="M6 10.5L9 13.5L14 8.5" stroke="white" strokeWidth="2" strokeLinecap="round"/>
                        </svg>
                    </span>
                    Apply
                </button>
            </div>
        </div>
    );
};
