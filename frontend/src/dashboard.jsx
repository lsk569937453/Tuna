import { Button, Card, Pagination } from "flowbite-react";
import { Table } from "flowbite-react";
import { HiOutlinePlus, HiSearch, HiShoppingCart } from "react-icons/hi";
import ReactECharts from 'echarts-for-react';
import Request from "./utils/axiosUtils";
import { useEffect, useState } from "react"; import BatteryGauge from 'react-battery-gauge'
import moment from 'moment';
import momentDurationPlugin from 'moment-duration-format'
momentDurationPlugin(moment)
const pageSize = 5;

function Dashboard() {

    const [dataPerMinute, setDataPerMinute] = useState([]);

    const [datePerDay, setDatePerDay] = useState([]);
    const [dataPerMinuteGroupByTaskId, setDataPerMinuteGroupByTaskId] = useState([]);
    const [dataPerdayGroupByTaskId, setDataPerdayGroupByTaskId] = useState([]);
    const [taskTableData, setTaskTableData] = useState([]);
    useEffect(() => {
        const interval = setInterval(() => {
            window.location.reload();
        }, 10000000); // 5000 milliseconds = 5 seconds

        return () => clearInterval(interval); // Cleanup on unmount
    }, []);
    useEffect(() => {
        getDataPerMinute();
        getDataPerDay();
        getDataPerMinuteGroupByTaskId();
        getDataPerDayGroupByTaskId();
        getSyncTaskSummary();
    }, []);

    const getDataPerMinute = () => {
        Request.get("/api/sqlLogs/perMinute").then((res) => {
            console.log(res);
            const mesArray = res.data.message.map(
                ({
                    minute: minute,
                    total_logs: total_logs,
                }) => {
                    return {
                        minute,
                        total_logs,
                    };
                }
            );
            setDataPerMinute(mesArray);
        });
    };
    const getSyncTaskSummary = () => {
        Request.get("/api/syncTaskLogs/summaryByTaskId").then((res) => {
            console.log(res);
            const mesArray = res.data.message.map(
                ({
                    sync_task_id: sync_task_id,
                    sync_task_uuid: sync_task_uuid,
                    latest_timestamp: latest_timestamp,
                    oldest_timestamp: oldest_timestamp,
                    online: online,
                    sync_task_name: sync_task_name,
                    duration_as_second: duration_as_second
                }) => {
                    return {
                        sync_task_id,
                        sync_task_uuid,
                        latest_timestamp,
                        oldest_timestamp,
                        online,
                        sync_task_name,
                        duration_as_second
                    };
                }
            );
            setTaskTableData(mesArray);
        });
    };
    const getDataPerDay = () => {
        Request.get("/api/sqlLogs/perDay").then((res) => {
            console.log(res);
            const mesArray = res.data.message.map(
                ({
                    day: day,
                    total_logs: total_logs,
                }) => {
                    return {
                        day,
                        total_logs,
                    };
                }
            );
            setDatePerDay(mesArray);
        });
    };
    const getDataPerMinuteGroupByTaskId = () => {
        Request.get("/api/sqlLogs/perMinuteTaskId").then((res) => {
            console.log(res);

            setDataPerMinuteGroupByTaskId(res.data.message);
        });
    };
    const getDataPerDayGroupByTaskId = () => {
        Request.get("/api/sqlLogs/perDayTaskId").then((res) => {
            console.log(res);

            setDataPerdayGroupByTaskId(res.data.message);
        });
    };
    const optionsForDataPerMinute = () => {


        return {
            title: {
                left: 'center',
                text: '今日同步总览'
            },
            xAxis: {
                type: 'category',
                data: dataPerMinute.map(item => item.minute)
            },
            yAxis: {
                name: '同步sql条数',

                type: 'value',

            },
            series: [
                {
                    data: dataPerMinute.map(item => item.total_logs),
                    type: 'bar'
                }
            ]
        };
    }

    const optionsForDataPerMinuteGroupByTaskId = () => {
        if (!dataPerMinuteGroupByTaskId || !dataPerMinuteGroupByTaskId.list || dataPerMinuteGroupByTaskId.list.length === 0) {
            return {

                xAxis: {
                    type: 'category',
                    data: []  // Empty x-axis data
                },
                yAxis: {
                    name: '同步sql条数',
                    type: 'value',
                },
                series: []  // No series data
            };
        }


        return {
            title: {

                text: '同步任务今日同步总览'

            },
            tooltip: {
                trigger: 'axis',
                axisPointer: {
                    type: 'cross',
                    label: {
                        backgroundColor: '#6a7985'
                    }
                }
            },
            legend: {
                data: dataPerMinuteGroupByTaskId.list.map(task => task.sync_task_name),
            },

            grid: {
                left: '3%',
                right: '4%',
                bottom: '3%',
                containLabel: true
            },
            xAxis: [
                {
                    type: 'time', boundaryGap: false,

                }
            ],
            yAxis: [
                {
                    name: '同步sql条数',

                    type: 'value', boundaryGap: [0, '100%'], min: 0.5


                }
            ],
            series: dataPerMinuteGroupByTaskId?.list.map(task => ({
                name: task.sync_task_name,  // Sync task name for each series
                data: task.total_logs,      // Corresponding logs data
                type: 'line',                // Type of chart (bar in this case)
                emphasis: {
                    focus: 'series'
                },
                showSymbol: false,
            }))
        };
    }
    const optionsForDataPerDay = () => {


        return {
            title: {
                left: 'center',
                text: '最近一个月同步总览'
            },
            xAxis: {
                type: 'category',
                data: datePerDay.map(item => item.day)
            },
            yAxis: {
                name: '同步sql条数',

                type: 'value',

            },
            series: [
                {
                    data: datePerDay.map(item => item.total_logs),
                    type: 'bar'
                }
            ]
        };
    }

    const optionsForDataPerDayGroupByTaskId = () => {
        if (!dataPerdayGroupByTaskId || !dataPerdayGroupByTaskId.all_days || !dataPerdayGroupByTaskId.list || dataPerdayGroupByTaskId.list.length === 0) {
            return {

                xAxis: {
                    type: 'category',
                    data: []  // Empty x-axis data
                },
                yAxis: {
                    name: '同步sql条数',
                    type: 'value',
                },
                series: []  // No series data
            };
        }


        return {
            title: {
                text: '同步任务今日同步总览'

            },
            tooltip: {
                trigger: 'axis',
                axisPointer: {
                    type: 'cross',
                    label: {
                        backgroundColor: '#6a7985'
                    }
                }
            },
            legend: {
                data: dataPerdayGroupByTaskId.list.map(task => task.sync_task_name),
            },

            grid: {
                left: '3%',
                right: '4%',
                bottom: '3%',
                containLabel: true
            },
            xAxis: [
                {
                    type: 'category',
                    boundaryGap: false,
                    data: dataPerdayGroupByTaskId.all_days
                }
            ],
            yAxis: [
                {
                    name: '同步sql条数',

                    type: 'value'
                }
            ],
            series: dataPerdayGroupByTaskId?.list.map(task => ({
                name: task.sync_task_name,  // Sync task name for each series
                data: task.total_logs,      // Corresponding logs data
                type: 'line',                // Type of chart (bar in this case)
                emphasis: {
                    focus: 'series'
                },
                // stack: 'Total',

            }))
        };
    }
    const showTimeDuration = (duration_as_second) => {
        if (!duration_as_second || typeof duration_as_second !== 'number') {
            return 'N/A';
        }
        console.log(duration_as_second);
        const a = moment
            .duration(duration_as_second, 'seconds')
            .format('h[小时], m[分钟], s[秒]');
        console.log(a);
        return a;
    }
    return (
        <div className="flex flex-col">
            <Card className="m-10 basis-12 	bg-slate-100	">
                <div className="flex flex-col overflow-auto	">
                    <h5 className="text-2xl font-bold tracking-tight text-gray-900 dark:text-white basis-12 text-center">
                        Tuna数据同步平台
                    </h5>
                    <div className="flex flex-row gap-4">
                        <div className="basis-1/3 bg-white	rounded-lg	p-4">
                            <ReactECharts option={optionsForDataPerMinute()} />

                        </div>
                        <div className="basis-1/3 bg-white	rounded-lg p-4 ">
                            <ReactECharts option={optionsForDataPerDay()} />

                        </div>
                        <div className="basis-1/3 bg-white	rounded-lg p-4 ">
                            <Table>
                                <Table.Head>
                                    <Table.HeadCell className="font-bold text-center text-xl">任务名称</Table.HeadCell>
                                    <Table.HeadCell className="font-bold text-center text-xl">状态</Table.HeadCell>
                                    <Table.HeadCell className="font-bold text-center text-xl">在线时间</Table.HeadCell>
                                    <Table.HeadCell className="font-bold text-center text-xl">开始时间</Table.HeadCell>

                                </Table.Head>
                                <Table.Body className="divide-y">
                                    {taskTableData.map((row, index) => (
                                        <Table.Row className="bg-white dark:border-gray-700 dark:bg-gray-800" key={index}>
                                            <Table.Cell className="whitespace-nowrap font-medium text-gray-900 dark:text-white text-center">
                                                {row.sync_task_name}
                                            </Table.Cell>
                                            <Table.Cell className="text-center flex justify-center">

                                                {row.online === 0 ? (
                                                    <svg className="stroke-green-500 icon icon-tabler icons-tabler-outline icon-tabler-wifi" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" ><path stroke="none" d="M0 0h24v24H0z" fill="none" /><path d="M12 18l.01 0" /><path d="M9.172 15.172a4 4 0 0 1 5.656 0" /><path d="M6.343 12.343a8 8 0 0 1 11.314 0" /><path d="M3.515 9.515c4.686 -4.687 12.284 -4.687 17 0" /></svg>
                                                )
                                                    :
                                                    (<svg className="stroke-red-500 icon icon-tabler icons-tabler-outline icon-tabler-wifi" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" ><path stroke="none" d="M0 0h24v24H0z" fill="none" /><path d="M12 18l.01 0" /><path d="M9.172 15.172a4 4 0 0 1 5.656 0" /><path d="M6.343 12.343a8 8 0 0 1 11.314 0" /><path d="M3.515 9.515c4.686 -4.687 12.284 -4.687 17 0" /></svg>
                                                    )}
                                            </Table.Cell>

                                            <Table.Cell className="text-center">
                                                {showTimeDuration(row.duration_as_second)}
                                            </Table.Cell>

                                            <Table.Cell className="text-center">
                                                {row.oldest_timestamp}
                                            </Table.Cell>
                                        </Table.Row>
                                    ))}

                                </Table.Body>
                            </Table>

                        </div>
                    </div>
                    <div className="flex flex-row  gap-4 pt-4">
                        <div className="basis-1/2 bg-white	rounded-lg p-4">
                            <ReactECharts option={optionsForDataPerMinuteGroupByTaskId()} />

                        </div>
                        <div className="basis-1/2 bg-white	rounded-lg p-4">
                            <ReactECharts option={optionsForDataPerDayGroupByTaskId()} />

                        </div>
                    </div>
                </div>
            </Card >

        </div >
    );
}
export default Dashboard;