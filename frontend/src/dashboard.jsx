import { Button, Card, Pagination } from "flowbite-react";
import { Table } from "flowbite-react";
import { HiOutlinePlus, HiSearch, HiShoppingCart } from "react-icons/hi";
import ReactECharts from 'echarts-for-react';
import Request from "./utils/axiosUtils";
import { useEffect, useState } from "react"; import BatteryGauge from 'react-battery-gauge'
import moment from 'moment';


const pageSize = 5;

function Dashboard() {
    const [tableData, setTableData] = useState([
        {
            name: "1#保温炉",
            realWeight: 100,
            realTemper: 0,
            upperWeight: 300,
            lowerWeight: 200,
            status: 0,
        },
    ]);
    const [agvInfo, setAgvInfo] = useState("");
    const [currentTableData, setCurrentTableData] = useState([]);
    const [totalTablePage, setTotalTablePage] = useState(1);
    const [currentPage, setCurrentPage] = useState(1);
    const [allTableData, setAllTableData] = useState([]);

    const [chart1Data, setChart1Data] = useState([]);

    useEffect(() => {
        getFurnaceData();
        searchAddWeight();
        getAgvInfo();
        const intervalId = setInterval(() => {
            getFurnaceData();
            getAgvInfo();
        }, 1000);
        return () => clearInterval(intervalId);
    }, []);

    useEffect(() => {
        const intervalId = setInterval(() => {
            searchAddWeight();
        }, 1000 * 60);
        return () => clearInterval(intervalId);

    }, []);
    const options1 = () => {
        const dates = [];
        const totalAddWeights = [];

        chart1Data.forEach((item) => {
            dates.push(item.date);
            totalAddWeights.push(item.totalAddWeight);
        });
        return {
            title: {
                left: 'center',
                bottom: 2,
                text: '每日加铝统计',
            },
            grid: { top: 8, right: 8, bottom: 48, left: 36 },
            xAxis: {
                type: 'category',
                data: dates,
            },
            yAxis: {
                type: 'value',
            },
            series: [
                {
                    data: totalAddWeights,
                    type: 'line',
                    smooth: true,
                },
            ],
            tooltip: {
                trigger: 'axis',
            },
        };
    }
    const onPageChange = (page) => {
        setCurrentPage(page);
        const currentTableData = allTableData.slice((page - 1) * pageSize, page * pageSize);

        setTableData(currentTableData);

    };
    const getFurnaceData = () => {
        Request.get("/api/getFurnaceInfor").then((res) => {
            console.log(res);
            const mesArray = res.data.message.map(
                ({
                    name: name,
                    status: status,
                    weight: weight

                }) => {
                    return {
                        name,
                        status,
                        weight
                    };
                }
            );
            const tdata = mesArray.slice(0, pageSize);

            setTableData(tdata);
            setAllTableData(mesArray);
            setTotalTablePage(Math.ceil(mesArray.length / pageSize));
        });
    };
    const options4 = () => {
        return {
            tooltip: {
                formatter: '{a} <br/>{b} : {c}m/s'
            },
            series: [
                {
                    name: '速度',
                    type: 'gauge',
                    min: 0,
                    max: 1,
                    detail: {
                        formatter: '{value}'
                    },
                    data: [
                        {
                            value: agvInfo === undefined ? 0 : agvInfo.velocity / 1000,
                            name: '速度'
                        }
                    ]
                }
            ]
        };
    }
    const searchAddWeight = () => {
        const currentDate = moment();
        const sevendDate = moment().add(-7, 'd');
        Request.post("/api/searchAddWeightLog", {
            endDate: currentDate.format("YYYY-MM-DD HH:mm:ss"),
            startDate: sevendDate.format("YYYY-MM-DD HH:mm:ss"),
            furnaceId: "1",
            offset: 0,
            limit: 1000,
        }).then((res) => {
            console.log(res);
            //按照每天的数据进行分组统计
            const groupedData = res.data.message.logData.reduce((acc, curr) => {
                const date = curr.timestamp.split('T')[0]; // Extract the date part
                if (!acc[date]) {
                    acc[date] = {
                        date: date,
                        totalAddWeight: 0
                    };
                }
                acc[date].totalAddWeight += curr.addWeight;
                return acc;
            }, {});

            const result = Object.values(groupedData);
            console.log(result);
            setChart1Data(result);


        });
    }
    const getAgvInfo = () => {

        Request.get("/api/getAgvInfor").then((res) => {
            console.log(res);

            setAgvInfo(res.data.message);

        });
    }

    const statusText = (status) => {
        if (status == 1) {
            return "缺铝";
        } else if (status == 2) {
            return "正常";
        } else if (status == 0) {
            return "离线";
        }

    }

    return (
        <div className="flex flex-col">
            <Card className="m-10 basis-12 	">
                <div className="flex flex-col overflow-auto	">
                    <h5 className="text-2xl font-bold tracking-tight text-gray-900 dark:text-white basis-12 text-center">
                        定量炉信息
                    </h5>
                    <div className="flex">
                        <div className="basis-1/2">
                            <ReactECharts option={options1()} />

                        </div>
                        <div className="basis-1/2 overflow-auto	">
                            <Table>
                                <Table.Head>
                                    <Table.HeadCell className="text-base">炉名</Table.HeadCell>

                                    <Table.HeadCell className="text-base">状态</Table.HeadCell>
                                    <Table.HeadCell className="text-base">加铝重量(Kg)</Table.HeadCell>

                                </Table.Head>
                                <Table.Body className="divide-y">
                                    {tableData.map((row, index) => (
                                        <Table.Row className="bg-white dark:border-gray-700 dark:bg-gray-800" key={index}>
                                            {/* <Table.Cell className="whitespace-nowrap font-medium text-gray-900 dark:text-white">
                                    {'Apple MacBook Pro 17"'}
                                </Table.Cell> */}
                                            <Table.Cell className="whitespace-nowrap font-medium text-gray-900 dark:text-white">{row.name}</Table.Cell>

                                            <Table.Cell>
                                                <div className="flex items-center gap-2">
                                                    {statusText(row.status)}
                                                    {row.status == 0 && <div className="bg-gray-400 w-20 h-8"></div>}
                                                    {row.status == 1 && <div className="bg-red-600 w-20 h-8"></div>}
                                                    {row.status == 2 && <div className="bg-green-600 w-20 h-8"></div>}

                                                </div>
                                            </Table.Cell>
                                            <Table.Cell>{row.weight}</Table.Cell>

                                        </Table.Row>
                                    ))}
                                </Table.Body>
                            </Table>
                            <Pagination currentPage={currentPage} totalPages={totalTablePage} onPageChange={onPageChange}
                                previousLabel="前一页"
                                nextLabel="后一页" />
                        </div>
                    </div>
                </div>
            </Card >
            <Card className="mx-10 basis-12">
                <div className="flex flex-col">
                    <h5 className="text-2xl font-bold tracking-tight text-gray-900 dark:text-white basis-12 text-center">
                        AGV信息
                    </h5>
                    <div className="flex">
                        <div className="basis-1/3 flex flex-col gap-5 border-dotted  border-r-2 border-indigo-600	">
                            <span className="font-bold text-black text-opacity-50">转运包铝水重量:</span>
                            {agvInfo && <span className="text-9xl font-bold				">{agvInfo.weight}Kg</span>}
                        </div>
                        <div className="basis-1/3 border-dotted  border-r-2 border-indigo-600">
                            <ReactECharts option={options4()} />
                        </div>
                        <div className="basis-1/3 flex justify-center items-center">
                            {agvInfo && <BatteryGauge value={agvInfo.battery} size={300} />}

                        </div>
                    </div>
                </div>
            </Card>
        </div >
    );
}
export default Dashboard;