import os
import re
DATA_DIR = 'test/data'
# files = os.listdir(DATA_DIR)
files = ['multi.stats',
         'D:stall_C:ANT.stats',
         'D:stall_C:Dyn1b.stats',
         'D:stall_C:Dyn2b.stats',
         'D:df_C:ANT.stats',
         'D:df_C:Dyn1b.stats',
         'D:df_C:Dyn2b.stats',
         ]

tables = {
    "ackermann": "",
    "add": "",
    "div": "",
    "dummy": "",
    "if-else": "",
    "load-store": "",
    "matrix-mul": "",
    "quicksort": "",
    "shift": "",
    "unalign": "",
}

# 定义正则表达式模式来匹配所需的数据
pattern = re.compile(
    r'-------Running (?P<test_name>[\w-]+)-------.*?'
    r'CPU run clock: (?P<run_clock>\d+).*?'
    r'CPU data hazard count: (?P<data_hazard_count>\d+).*?'
    r'CPU data hazard delayed cycles: (?P<data_hazard_delayed_cycles>\d+).*?'
    r'CPU control hazard count: (?P<control_hazard_count>\d+).*?'
    r'CPU control hazard delayed cycles: (?P<control_hazard_delayed_cycles>\d+).*?'
    r'CPU executed valid instructions: (?P<valid_instructions>\d+).*?'
    r'CPI = (?P<CPI>[\d.]+)',
    re.DOTALL
)


def process_file(file_name: str):
    config_name = file_name.split('.')[0]
    f = open(file=f"{DATA_DIR}/{file_name}", mode='r')
    data = f.read()

    matches = pattern.finditer(data)
    # 提取每个匹配的数据
    extracted_data = []
    for match in matches:
        extracted_data.append(match.groupdict())

    # 打印提取的数据
    for entry in extracted_data:
        print(entry)
        """
        {
            'test_name': 'load-store', 
            'run_clock': '425', 
            'data_hazard_count': '151', 
            'data_hazard_delayed_cycles': '0', 
            'control_hazard_count': '17', 
            'control_hazard_delayed_cycles': '34', 
            'valid_instructions': '390',
            'CPI': '1.0897435897435896'
        }
        """
        s = tables[entry['test_name']]
        s += f"| {config_name} | {entry['run_clock']} | {entry['valid_instructions']} | {float(entry['CPI']):.3f} | {entry['data_hazard_count']} | {entry['data_hazard_delayed_cycles']} | {entry['control_hazard_count']} | {entry['control_hazard_delayed_cycles']} |\n"
        tables[entry['test_name']] = s
    f.close()


for (key, s) in tables.items():
    title = f"| {key}  | clock | insts | CPI | DH count | DH cycles | CH count | CH cycles |\n"
    split_line = "|:------------------|-------|:------|:----|----------|:----------|:---------|:----------|\n"
    tables[key] = s + title + split_line

for file_name in files:
    process_file(file_name)

print(tables)

with open('REPORT.md', 'w') as wf:
    for (_, s) in tables.items():
        wf.write('\n')
        wf.write(s)

"""
| Multi             |   1565    |   389    |  4.02   |    0      |    0       |      0    |      0     |
| D:stall C:ANT     |       |       |     |          |           |          |           |
| D:stall C: Dyn 1b |       |       |     |          |           |          |           |
| D:stall C: Dyn 2b |       |       |     |          |           |          |           |
| D:df C:ANT        |       |       |     |          |           |          |           |
| D:df C: Dyn 1b    |       |       |     |          |           |          |           |
| D:df C: Dyn 2b    |       |       |     |          |           |          |           |
"""

if __name__ == '__main__':
    print(files)
